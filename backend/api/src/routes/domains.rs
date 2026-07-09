use axum::{
    routing::{get, post, delete},
    Router,
    Json,
    http::{StatusCode, HeaderMap},
    Extension,
    extract::Path,
};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use super::auth_helper::authenticate_project;

#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub domain: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_domains_handler).post(create_domain_handler))
        .route("/:id", delete(delete_domain_handler))
        .route("/:id/verify", post(verify_domain_handler))
}

async fn list_domains_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    match db::find_domains_by_project(&pool, &project.id).await {
        Ok(domains) => (StatusCode::OK, Json(json!(domains))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn get_ses_client() -> Option<aws_sdk_sesv2::Client> {
    if std::env::var("AWS_ACCESS_KEY_ID").is_ok() && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest()).load().await;
        Some(aws_sdk_sesv2::Client::new(&config))
    } else {
        None
    }
}

async fn create_domain_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateDomainRequest>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    let domain_name = payload.domain.trim().to_lowercase();
    if domain_name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Domain name is required" })));
    }

    let id = Uuid::new_v4().to_string();

    let mut dkim_tokens = json!([
        format!("fm1._domainkey.{}", domain_name),
        format!("fm2._domainkey.{}", domain_name)
    ]);

    if let Some(client) = get_ses_client().await {
        match client.create_email_identity().email_identity(&domain_name).send().await {
            Ok(res) => {
                if let Some(dkim_attributes) = res.dkim_attributes() {
                    let tokens = dkim_attributes.tokens();
                    let tokens_json: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
                    dkim_tokens = json!(tokens_json);
                }
            }
            Err(_) => {
                if let Ok(res) = client.get_email_identity().email_identity(&domain_name).send().await {
                    if let Some(dkim_attributes) = res.dkim_attributes() {
                        let tokens = dkim_attributes.tokens();
                        let tokens_json: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
                        dkim_tokens = json!(tokens_json);
                    }
                }
            }
        }
    }

    match db::create_domain(&pool, &id, &project.id, &domain_name, dkim_tokens).await {
        Ok(domain) => (StatusCode::CREATED, Json(json!(domain))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}


async fn delete_domain_handler(
    headers: HeaderMap,
    Path(domain_id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    match db::delete_domain(&pool, &project.id, &domain_id).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "success": true }))),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Domain not found" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn verify_domain_handler(
    headers: HeaderMap,
    Path(domain_id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let _project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    // 1. Fetch domain record
    let domain_opt = match db::find_domain_by_id(&pool, &domain_id).await {
        Ok(Some(d)) => d,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Domain not found" }))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    };

    let lookup_target = format!("ferromail-verify.{}", domain_opt.domain);
    let expected_txt_value = format!("ferromail-verification={}", domain_opt.id);

    // 2. Perform DNS Lookup or AWS SES Verification Check
    let is_verified = if let Some(client) = get_ses_client().await {
        match client.get_email_identity().email_identity(&domain_opt.domain).send().await {
            Ok(res) => res.verified_for_sending_status(),
            Err(_) => false,
        }
    } else {
        let is_local_or_mock = domain_opt.domain == "localhost" 
            || domain_opt.domain.ends_with(".local") 
            || domain_opt.domain.ends_with(".test")
            || domain_opt.domain == "example.com";

        if is_local_or_mock {
            true
        } else {
            match hickory_resolver::TokioAsyncResolver::tokio_from_system_conf() {
                Ok(resolver) => {
                    match resolver.txt_lookup(lookup_target.clone()).await {
                        Ok(txt_lookup) => {
                            let mut found = false;
                            for txt in txt_lookup.iter() {
                                for data in txt.iter() {
                                    if let Ok(s) = std::str::from_utf8(data) {
                                        if s.trim() == expected_txt_value {
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                                if found { break; }
                            }
                            found
                        }
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }
    };

    if !is_verified {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Verification failed. DNS TXT record not found, does not match, or AWS SES identity pending.",
                "expected_host": lookup_target,
                "expected_value": expected_txt_value
            })),
        );
    }

    // 3. Update verification state in Database
    match db::update_domain_verified(&pool, &domain_id, true).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "success": true, "verified": true, "message": "DNS records successfully verified." }))),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Domain not found or unable to verify" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

