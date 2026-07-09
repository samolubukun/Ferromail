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
pub struct CreateContactRequest {
    pub email: String,
    pub data: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct CsvImportRequest {
    pub contacts: Vec<CreateContactRequest>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_contacts_handler).post(create_contact_handler))
        .route("/:id", delete(delete_contact_handler))
        .route("/csv", post(import_csv_handler))
}

async fn list_contacts_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    match db::find_contacts_by_project(&pool, &project.id).await {
        Ok(contacts) => (StatusCode::OK, Json(json!(contacts))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn create_contact_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateContactRequest>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    if payload.email.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Email is required" })));
    }

    let contact_id = Uuid::new_v4().to_string();

    match db::create_contact(&pool, &contact_id, &payload.email, &project.id, payload.data).await {
        Ok(contact) => (StatusCode::CREATED, Json(json!(contact))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn delete_contact_handler(
    headers: HeaderMap,
    Path(contact_id): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    match db::delete_contact(&pool, &project.id, &contact_id).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "success": true }))),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn import_csv_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CsvImportRequest>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    let mut imported_count = 0;

    // Use a transaction for bulk inserting
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Tx begin error: {:?}", e) }))),
    };

    for contact_req in payload.contacts {
        if contact_req.email.trim().is_empty() {
            continue;
        }

        let contact_id = Uuid::new_v4().to_string();
        let query_res = sqlx::query(
            "INSERT INTO contacts (id, email, data, subscribed, \"projectId\", \"createdAt\", \"updatedAt\")
             VALUES ($1, $2, $3, true, $4, NOW(), NOW())
             ON CONFLICT (\"projectId\", email) DO UPDATE
             SET data = EXCLUDED.data, \"updatedAt\" = NOW()"
        )
        .bind(&contact_id)
        .bind(&contact_req.email)
        .bind(contact_req.data.unwrap_or(json!({})))
        .bind(&project.id)
        .execute(&mut *tx)
        .await;

        if query_res.is_ok() {
            imported_count += 1;
        }
    }

    if let Err(e) = tx.commit().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Tx commit error: {:?}", e) })));
    }

    (StatusCode::OK, Json(json!({ "success": true, "imported": imported_count })))
}
