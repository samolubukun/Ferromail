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
use super::auth_helper::authenticate_project;

#[derive(Debug, Deserialize)]
pub struct WorkflowStepInput {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub step_type: String, // TRIGGER, SEND_EMAIL, DELAY, WEBHOOK, etc.
    pub name: String,
    pub position: Value,
    pub config: Value,
    #[serde(rename = "templateId")]
    pub template_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowTransitionInput {
    #[serde(rename = "fromStepId")]
    pub from_step_id: String,
    #[serde(rename = "toStepId")]
    pub to_step_id: String,
    pub condition: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "triggerType")]
    pub trigger_type: String, // EVENT, MANUAL, SCHEDULE
    #[serde(rename = "triggerConfig")]
    pub trigger_config: Option<Value>,
    #[serde(rename = "allowReentry")]
    pub allow_reentry: Option<bool>,
    pub steps: Vec<WorkflowStepInput>,
    pub transitions: Vec<WorkflowTransitionInput>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_workflows_handler).post(create_workflow_handler))
}

async fn list_workflows_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    match db::find_workflows_by_project(&pool, &project.id).await {
        Ok(workflows) => (StatusCode::OK, Json(json!(workflows))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("{:?}", e) }))),
    }
}

async fn create_workflow_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> (StatusCode, Json<Value>) {
    let project = match authenticate_project(&headers, &pool).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNAUTHORIZED, Json(json!({ "error": e }))),
    };

    if payload.name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Workflow name is required" })));
    }

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Transaction start failed: {:?}", e) }))),
    };

    let workflow_id = Uuid::new_v4().to_string();

    let trigger = match payload.trigger_type.to_uppercase().as_str() {
        "MANUAL" => db::models::WorkflowTriggerType::Manual,
        "SCHEDULE" => db::models::WorkflowTriggerType::Schedule,
        _ => db::models::WorkflowTriggerType::Event,
    };

    // 1. Insert core Workflow
    let workflow_res = sqlx::query(
        "INSERT INTO workflows (id, name, description, enabled, \"triggerType\", \"triggerConfig\", \"allowReentry\", \"projectId\", \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, false, $4, $5, $6, $7, NOW(), NOW())"
    )
    .bind(&workflow_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(trigger)
    .bind(payload.trigger_config.unwrap_or(json!({})))
    .bind(payload.allow_reentry.unwrap_or(false))
    .bind(&project.id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = workflow_res {
        let _ = tx.rollback().await;
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to create workflow: {:?}", e) })));
    }

    // 2. Insert Workflow Steps
    for step in payload.steps {
        let step_id = step.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let step_type = match step.step_type.to_uppercase().as_str() {
            "TRIGGER" => db::models::WorkflowStepType::Trigger,
            "SEND_EMAIL" => db::models::WorkflowStepType::SendEmail,
            "DELAY" => db::models::WorkflowStepType::Delay,
            "WAIT_FOR_EVENT" => db::models::WorkflowStepType::WaitForEvent,
            "CONDITION" => db::models::WorkflowStepType::Condition,
            "EXIT" => db::models::WorkflowStepType::Exit,
            "WEBHOOK" => db::models::WorkflowStepType::Webhook,
            _ => db::models::WorkflowStepType::UpdateContact,
        };

        let step_res = sqlx::query(
            "INSERT INTO workflow_steps (id, type, name, position, config, \"workflowId\", \"templateId\", \"createdAt\", \"updatedAt\")
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())"
        )
        .bind(&step_id)
        .bind(step_type)
        .bind(&step.name)
        .bind(&step.position)
        .bind(&step.config)
        .bind(&workflow_id)
        .bind(&step.template_id)
        .execute(&mut *tx)
        .await;

        if let Err(e) = step_res {
            let _ = tx.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to create workflow step: {:?}", e) })));
        }
    }

    // 3. Insert Workflow Transitions
    for trans in payload.transitions {
        let trans_id = Uuid::new_v4().to_string();

        let trans_res = sqlx::query(
            "INSERT INTO workflow_transitions (id, \"fromStepId\", \"toStepId\", condition, priority, \"createdAt\", \"updatedAt\")
             VALUES ($1, $2, $3, $4, 0, NOW(), NOW())"
        )
        .bind(&trans_id)
        .bind(&trans.from_step_id)
        .bind(&trans.to_step_id)
        .bind(trans.condition)
        .execute(&mut *tx)
        .await;

        if let Err(e) = trans_res {
            let _ = tx.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to create workflow transition: {:?}", e) })));
        }
    }

    if let Err(e) = tx.commit().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to commit workflow transaction: {:?}", e) })));
    }

    (StatusCode::CREATED, Json(json!({ "success": true, "workflowId": workflow_id })))
}
