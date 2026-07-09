use axum::{
    routing::{get, post},
    Router,
    Json,
    http::{StatusCode, HeaderMap},
    Extension,
};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::auth_helper::authenticate_project;

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub description: Option<String>,
    pub subject: String,
    pub body: String,
    pub from: String,
    #[serde(rename = "fromName")]
    pub from_name: Option<String>,
    #[serde(rename = "replyTo")]
    pub reply_to: Option<String>,
    #[serde(rename = "audienceType")]
    pub audience_type: String, // ALL, SEGMENT, FILTERED
    #[serde(rename = "segmentId")]
    pub segment_id: Option<String>,
    #[serde(rename = "scheduledFor")]
    pub scheduled_for: Option<DateTime<Utc>>,
    #[serde(rename = "sendImmediately")]
    pub send_immediately: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_campaigns_handler).post(create_campaign_handler))
}

async fn list_campaigns_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    match db::find_campaigns_by_project(&pool, &project.id).await {
        Ok(campaigns) => (StatusCode::OK, Json(json!(campaigns))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn create_campaign_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateCampaignRequest>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    if payload.name.trim().is_empty() || payload.subject.trim().is_empty() || payload.body.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Name, subject, and body are required fields." })));
    }

    let id = Uuid::new_v4().to_string();

    let aud_type = match payload.audience_type.to_uppercase().as_str() {
        "SEGMENT" => db::models::CampaignAudienceType::Segment,
        "FILTERED" => db::models::CampaignAudienceType::Filtered,
        _ => db::models::CampaignAudienceType::All,
    };

    // If sendImmediately is true, we set total_recipients sentinel to -99 so the background scheduler catches it
    let total_recipients = if payload.send_immediately.unwrap_or(false) {
        -99
    } else {
        0
    };

    let scheduled_time = if payload.send_immediately.unwrap_or(false) {
        None
    } else {
        payload.scheduled_for
    };

    let res = sqlx::query_as::<_, db::models::Campaign>(
        "INSERT INTO campaigns (
            id, name, description, status, subject, body, \"from\", \"fromName\", \"replyTo\", type,
            \"audienceType\", \"audienceCondition\", \"segmentId\", \"scheduledFor\", \"totalRecipients\",
            \"sentCount\", \"deliveredCount\", \"openedCount\", \"clickedCount\", \"bouncedCount\", \"projectId\",
            \"createdAt\", \"updatedAt\"
         )
         VALUES (
            $1, $2, $3, 'DRAFT', $4, $5, $6, $7, $8, 'MARKETING',
            $9, NULL, $10, $11, $12,
            0, 0, 0, 0, 0, $13,
            NOW(), NOW()
         )
         RETURNING *"
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.subject)
    .bind(&payload.body)
    .bind(&payload.from)
    .bind(&payload.from_name)
    .bind(&payload.reply_to)
    .bind(aud_type)
    .bind(&payload.segment_id)
    .bind(scheduled_time)
    .bind(total_recipients)
    .bind(&project.id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(campaign) => {
            // If it should send immediately, set status to SCHEDULED so background thread picks it up
            if payload.send_immediately.unwrap_or(false) {
                let _ = sqlx::query("UPDATE campaigns SET status = 'SCHEDULED' WHERE id = $1")
                    .bind(&campaign.id)
                    .execute(&pool)
                    .await;
            }
            (StatusCode::CREATED, Json(json!(campaign)))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}
