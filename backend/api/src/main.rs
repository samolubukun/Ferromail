use std::env;
use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
    Json,
    http::{StatusCode, HeaderMap},
    Extension,
    extract::{Path, Query},
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use sqlx::{PgPool, Transaction, Postgres};
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};
use uuid::Uuid;

pub mod routes {
    pub mod auth;
    pub mod auth_helper;
    pub mod contacts;
    pub mod domains;
    pub mod campaigns;
    pub mod workflows;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting Ferromail HTTP API...");

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    
    let db_pool = db::establish_connection(&database_url).await?;
    info!("Database connection established.");

    // CORS Setup
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    // Routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/v1/send", post(send_email_route))
        .route("/v1/track/open/:email_id", get(track_open_route))
        .route("/v1/track/click/:email_id", get(track_click_route))
        .nest("/v1/auth", routes::auth::router())
        .nest("/v1/contacts", routes::contacts::router())
        .nest("/v1/domains", routes::domains::router())
        .nest("/v1/campaigns", routes::campaigns::router())
        .nest("/v1/workflows", routes::workflows::router())
        .layer(cors)
        .layer(Extension(db_pool));

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    info!("API server listening on {}...", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Health check handler
async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "time": chrono::Utc::now().timestamp_millis(),
            "service": "Ferromail HTTP API"
        })),
    )
}

// Send Email DTOs
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FromField {
    String(String),
    Object { name: String, email: String },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ToField {
    String(String),
    ArrayOfStrings(Vec<String>),
    ArrayOfObjects(Vec<RecipientObject>),
}

#[derive(Debug, Deserialize)]
struct RecipientObject {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct SendEmailRequest {
    from: FromField,
    to: ToField,
    subject: String,
    body: String,
    headers: Option<serde_json::Value>,
    attachments: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct SendEmailResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email_ids: Option<Vec<String>>,
}

fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() {
        return false;
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }
    true
}

// Route to handle email sending requests
async fn send_email_route(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<SendEmailRequest>,
) -> (StatusCode, Json<SendEmailResponse>) {
    // 1. Authenticate Request using transparent auth helper
    let project = match routes::auth_helper::authenticate_project(&headers, &pool).await {
        Ok(proj) => proj,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SendEmailResponse {
                    success: false,
                    message: format!("Authentication failed: {}", e),
                    email_ids: None,
                }),
            );
        }
    };

    // 2. Parse recipients and sender
    let from_email = match &payload.from {
        FromField::String(s) => s.clone(),
        FromField::Object { email, .. } => email.clone(),
    };

    if !is_valid_email(&from_email) {
        return (
            StatusCode::BAD_REQUEST,
            Json(SendEmailResponse {
                success: false,
                message: format!("Invalid sender email address: {}", from_email),
                email_ids: None,
            }),
        );
    }

    let from_name = match &payload.from {
        FromField::String(_) => None,
        FromField::Object { name, .. } => Some(name.clone()),
    };

    let recipients = match &payload.to {
        ToField::String(s) => vec![RecipientObject {
            name: String::new(),
            email: s.clone(),
        }],
        ToField::ArrayOfStrings(arr) => arr
            .iter()
            .map(|s| RecipientObject {
                name: String::new(),
                email: s.clone(),
            })
            .collect(),
        ToField::ArrayOfObjects(arr) => arr
            .iter()
            .map(|obj| RecipientObject {
                name: obj.name.clone(),
                email: obj.email.clone(),
            })
            .collect(),
    };

    if recipients.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(SendEmailResponse {
                success: false,
                message: "Recipient list is empty".to_string(),
                email_ids: None,
            }),
        );
    }

    for recipient in &recipients {
        if !is_valid_email(&recipient.email) {
            return (
                StatusCode::BAD_REQUEST,
                Json(SendEmailResponse {
                    success: false,
                    message: format!("Invalid recipient email address: {}", recipient.email),
                    email_ids: None,
                }),
            );
        }
    }

    // 3. Process in Database transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to start transaction: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SendEmailResponse {
                    success: false,
                    message: "Transaction initialization failed".to_string(),
                    email_ids: None,
                }),
            );
        }
    };

    let mut email_ids = Vec::new();

    for recipient in recipients {
        let contact_id = match get_or_create_contact(&mut tx, &project.id, &recipient.email).await {
            Ok(id) => id,
            Err(e) => {
                error!("Error getting or creating contact: {:?}", e);
                let _ = tx.rollback().await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SendEmailResponse {
                        success: false,
                        message: "Database contact processing error".to_string(),
                        email_ids: None,
                    }),
                );
            }
        };

        // Create Email Record
        let email_id = Uuid::new_v4().to_string();
        let headers_json = payload.headers.clone().unwrap_or(json!({}));
        let attachments_json = payload.attachments.clone().unwrap_or(json!([]));

        let insert_res = sqlx::query(
            "INSERT INTO emails (
                id, \"contactId\", \"toName\", subject, body, \"from\", \"fromName\",
                headers, attachments, \"sourceType\", status, \"projectId\",
                \"createdAt\", \"updatedAt\"
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'TRANSACTIONAL', 'PENDING', $10, NOW(), NOW())"
        )
        .bind(&email_id)
        .bind(&contact_id)
        .bind(if recipient.name.is_empty() { None } else { Some(recipient.name.clone()) })
        .bind(&payload.subject)
        .bind(&payload.body)
        .bind(&from_email)
        .bind(&from_name)
        .bind(headers_json)
        .bind(attachments_json)
        .bind(&project.id)
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert_res {
            error!("Failed to insert email record: {:?}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SendEmailResponse {
                    success: false,
                    message: "Database email logging error".to_string(),
                    email_ids: None,
                }),
            );
        }

        email_ids.push(email_id);
    }

    if let Err(e) = tx.commit().await {
        error!("Failed to commit database transaction: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SendEmailResponse {
                success: false,
                message: "Database transaction commit error".to_string(),
                email_ids: None,
            }),
        );
    }

    info!(
        "Emails queued for delivery: count={}, project_id={}",
        email_ids.len(),
        project.id
    );

    (
        StatusCode::OK,
        Json(SendEmailResponse {
            success: true,
            message: "Emails queued for delivery successfully".to_string(),
            email_ids: Some(email_ids),
        }),
    )
}

// Look up or create a contact in a transaction
async fn get_or_create_contact(
    tx: &mut Transaction<'_, Postgres>,
    project_id: &str,
    email: &str,
) -> Result<String, sqlx::Error> {
    // Check if contact already exists
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM contacts WHERE \"projectId\" = $1 AND email = $2 LIMIT 1"
    )
    .bind(project_id)
    .bind(email)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    // Create a new contact
    let new_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO contacts (id, email, \"projectId\", subscribed, \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, true, NOW(), NOW())"
    )
    .bind(&new_id)
    .bind(email)
    .bind(project_id)
    .execute(&mut **tx)
    .await?;

    Ok(new_id)
}

const TRANSPARENT_1X1_GIF: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];

async fn track_open_route(
    Path(email_id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let select_res: Option<(String, String, Option<String>, Option<chrono::NaiveDateTime>)> = match sqlx::query_as(
        "SELECT \"projectId\", \"contactId\", \"campaignId\", \"openedAt\" FROM emails WHERE id = $1 LIMIT 1"
    )
    .bind(&email_id)
    .fetch_optional(&pool)
    .await {
        Ok(opt) => opt,
        Err(_) => None,
    };

    if let Some((project_id, contact_id, campaign_id, opened_at)) = select_res {
        let tx = match pool.begin().await {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        if let Some(mut transaction) = tx {
            let mut success = true;
            if opened_at.is_none() {
                // First open
                let update_email = sqlx::query(
                    "UPDATE emails 
                     SET status = 'OPENED', 
                         \"openedAt\" = NOW(), 
                         opens = opens + 1, 
                         \"updatedAt\" = NOW() 
                     WHERE id = $1 AND \"openedAt\" IS NULL"
                )
                .bind(&email_id)
                .execute(&mut *transaction)
                .await;

                if let Ok(res) = update_email {
                    if res.rows_affected() > 0 {
                        // Create event log
                        let event_id = Uuid::new_v4().to_string();
                        let event_log = sqlx::query(
                            "INSERT INTO events (id, name, data, \"projectId\", \"contactId\", \"emailId\", \"createdAt\")
                             VALUES ($1, 'email.opened', $2, $3, $4, $5, NOW())"
                        )
                        .bind(&event_id)
                        .bind(json!({ "openedAt": chrono::Utc::now().to_rfc3339() }))
                        .bind(&project_id)
                        .bind(&contact_id)
                        .bind(&email_id)
                        .execute(&mut *transaction)
                        .await;

                        if event_log.is_err() { success = false; }

                        // Update campaign count if exists
                        if let Some(camp_id) = campaign_id {
                            let update_camp = sqlx::query(
                                "UPDATE campaigns SET \"openedCount\" = \"openedCount\" + 1 WHERE id = $1"
                            )
                            .bind(&camp_id)
                            .execute(&mut *transaction)
                            .await;

                            if update_camp.is_err() { success = false; }
                        }
                    }
                } else {
                    success = false;
                }
            } else {
                // Subsequent open
                let update_email = sqlx::query(
                    "UPDATE emails SET opens = opens + 1, \"updatedAt\" = NOW() WHERE id = $1"
                )
                .bind(&email_id)
                .execute(&mut *transaction)
                .await;

                if update_email.is_err() { success = false; }
            }

            if success {
                let _ = transaction.commit().await;
            } else {
                let _ = transaction.rollback().await;
            }
        }
    }

    // Return transparent GIF
    let headers = [
        ("Content-Type", "image/gif"),
        ("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0"),
        ("Pragma", "no-cache"),
        ("Expires", "0"),
    ];

    (StatusCode::OK, headers, axum::body::Bytes::from(TRANSPARENT_1X1_GIF))
}

#[derive(Debug, Deserialize)]
struct ClickQueryParams {
    url: Option<String>,
}

async fn track_click_route(
    Path(email_id): Path<String>,
    Query(params): Query<ClickQueryParams>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let target_url = params.url.unwrap_or_else(|| "/".to_string());

    let select_res: Option<(String, String, Option<String>, Option<chrono::NaiveDateTime>)> = match sqlx::query_as(
        "SELECT \"projectId\", \"contactId\", \"campaignId\", \"clickedAt\" FROM emails WHERE id = $1 LIMIT 1"
    )
    .bind(&email_id)
    .fetch_optional(&pool)
    .await {
        Ok(opt) => opt,
        Err(_) => None,
    };

    if let Some((project_id, contact_id, campaign_id, clicked_at)) = select_res {
        let tx = match pool.begin().await {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        if let Some(mut transaction) = tx {
            let mut success = true;
            if clicked_at.is_none() {
                // First click
                let update_email = sqlx::query(
                    "UPDATE emails 
                     SET status = 'CLICKED', 
                         \"clickedAt\" = NOW(), 
                         clicks = clicks + 1, 
                         \"updatedAt\" = NOW() 
                     WHERE id = $1 AND \"clickedAt\" IS NULL"
                )
                .bind(&email_id)
                .execute(&mut *transaction)
                .await;

                if let Ok(res) = update_email {
                    if res.rows_affected() > 0 {
                        // Create event log
                        let event_id = Uuid::new_v4().to_string();
                        let event_log = sqlx::query(
                            "INSERT INTO events (id, name, data, \"projectId\", \"contactId\", \"emailId\", \"createdAt\")
                             VALUES ($1, 'email.clicked', $2, $3, $4, $5, NOW())"
                        )
                        .bind(&event_id)
                        .bind(json!({ "clickedAt": chrono::Utc::now().to_rfc3339(), "url": target_url }))
                        .bind(&project_id)
                        .bind(&contact_id)
                        .bind(&email_id)
                        .execute(&mut *transaction)
                        .await;

                        if event_log.is_err() { success = false; }

                        // Update campaign count if exists
                        if let Some(camp_id) = campaign_id {
                            let update_camp = sqlx::query(
                                "UPDATE campaigns SET \"clickedCount\" = \"clickedCount\" + 1 WHERE id = $1"
                            )
                            .bind(&camp_id)
                            .execute(&mut *transaction)
                            .await;

                            if update_camp.is_err() { success = false; }
                        }
                    }
                } else {
                    success = false;
                }
            } else {
                // Subsequent click
                let update_email = sqlx::query(
                    "UPDATE emails SET clicks = clicks + 1, \"updatedAt\" = NOW() WHERE id = $1"
                )
                .bind(&email_id)
                .execute(&mut *transaction)
                .await;

                if update_email.is_err() { success = false; }
            }

            if success {
                let _ = transaction.commit().await;
            } else {
                let _ = transaction.rollback().await;
            }
        }
    }

    let mut response = axum::response::Redirect::to(&target_url).into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    response
}
