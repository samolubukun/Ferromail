use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use serde_json::Value;

// ============================================
// ENUMS (PostgreSQL Enums)
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "ProjectDisabledReason", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectDisabledReason {
    PaymentFailed,
    EmailReputation,
    PhishingDetected,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "AuthMethod", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthMethod {
    Password,
    GoogleOauth,
    GithubOauth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "Role", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TemplateType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TemplateType {
    Transactional,
    Marketing,
    Headless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TrackingMode", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingMode {
    Enabled,
    Disabled,
    MarketingOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "CampaignStatus", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "CampaignAudienceType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CampaignAudienceType {
    All,
    Filtered,
    Segment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "WorkflowTriggerType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowTriggerType {
    Event,
    Manual,
    Schedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "WorkflowStepType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowStepType {
    Trigger,
    SendEmail,
    Delay,
    WaitForEvent,
    Condition,
    Exit,
    Webhook,
    UpdateContact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "WorkflowExecutionStatus", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowExecutionStatus {
    Running,
    Waiting,
    Completed,
    Exited,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "StepExecutionStatus", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepExecutionStatus {
    Pending,
    Scheduled,
    Waiting,
    Running,
    Completed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "SegmentType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SegmentType {
    Dynamic,
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "EmailSourceType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmailSourceType {
    Transactional,
    Campaign,
    Workflow,
    Inbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "EmailStatus", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmailStatus {
    Pending,
    Sending,
    Sent,
    Delivered,
    Received,
    Opened,
    Clicked,
    Bounced,
    Complained,
    Failed,
}

// ============================================
// STRUCTS (PostgreSQL Tables)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: Option<String>,
    #[sqlx(rename = "type")]
    pub auth_type: AuthMethod,
    #[sqlx(rename = "emailVerified")]
    pub email_verified: bool,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub public: String,
    pub secret: String,
    pub disabled: bool,
    #[sqlx(rename = "disabledReason")]
    pub disabled_reason: Option<ProjectDisabledReason>,
    pub customer: Option<String>,
    pub subscription: Option<String>,
    #[sqlx(rename = "billingLimitWorkflows")]
    pub billing_limit_workflows: Option<i32>,
    #[sqlx(rename = "billingLimitCampaigns")]
    pub billing_limit_campaigns: Option<i32>,
    #[sqlx(rename = "billingLimitTransactional")]
    pub billing_limit_transactional: Option<i32>,
    #[sqlx(rename = "billingLimitInbound")]
    pub billing_limit_inbound: Option<i32>,
    pub tracking: TrackingMode,
    pub language: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Membership {
    #[sqlx(rename = "userId")]
    pub user_id: String,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    pub role: Role,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub id: String,
    pub domain: String,
    pub verified: bool,
    #[sqlx(rename = "dkimTokens")]
    pub dkim_tokens: Option<Value>,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contact {
    pub id: String,
    pub email: String,
    pub data: Option<Value>,
    pub subscribed: bool,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub subject: String,
    pub body: String,
    pub from: String,
    #[sqlx(rename = "fromName")]
    pub from_name: Option<String>,
    #[sqlx(rename = "replyTo")]
    pub reply_to: Option<String>,
    #[sqlx(rename = "type")]
    pub template_type: TemplateType,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(rename = "type")]
    pub segment_type: SegmentType,
    pub condition: Option<Value>,
    #[sqlx(rename = "trackMembership")]
    pub track_membership: bool,
    #[sqlx(rename = "memberCount")]
    pub member_count: i32,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SegmentMembership {
    #[sqlx(rename = "contactId")]
    pub contact_id: String,
    #[sqlx(rename = "segmentId")]
    pub segment_id: String,
    #[sqlx(rename = "enteredAt")]
    pub entered_at: NaiveDateTime,
    #[sqlx(rename = "exitedAt")]
    pub exited_at: Option<NaiveDateTime>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: CampaignStatus,
    pub subject: String,
    pub body: String,
    pub from: String,
    #[sqlx(rename = "fromName")]
    pub from_name: Option<String>,
    #[sqlx(rename = "replyTo")]
    pub reply_to: Option<String>,
    #[sqlx(rename = "type")]
    pub template_type: TemplateType,
    #[sqlx(rename = "audienceType")]
    pub audience_type: CampaignAudienceType,
    #[sqlx(rename = "audienceCondition")]
    pub audience_condition: Option<Value>,
    #[sqlx(rename = "segmentId")]
    pub segment_id: Option<String>,
    #[sqlx(rename = "scheduledFor")]
    pub scheduled_for: Option<NaiveDateTime>,
    #[sqlx(rename = "totalRecipients")]
    pub total_recipients: i32,
    #[sqlx(rename = "sentCount")]
    pub sent_count: i32,
    #[sqlx(rename = "deliveredCount")]
    pub delivered_count: i32,
    #[sqlx(rename = "openedCount")]
    pub opened_count: i32,
    #[sqlx(rename = "clickedCount")]
    pub clicked_count: i32,
    #[sqlx(rename = "bouncedCount")]
    pub bounced_count: i32,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "sentAt")]
    pub sent_at: Option<NaiveDateTime>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    #[sqlx(rename = "triggerType")]
    pub trigger_type: WorkflowTriggerType,
    #[sqlx(rename = "triggerConfig")]
    pub trigger_config: Option<Value>,
    #[sqlx(rename = "allowReentry")]
    pub allow_reentry: bool,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowStep {
    pub id: String,
    #[sqlx(rename = "type")]
    pub step_type: WorkflowStepType,
    pub name: String,
    pub position: Value,
    pub config: Value,
    #[sqlx(rename = "workflowId")]
    pub workflow_id: String,
    #[sqlx(rename = "templateId")]
    pub template_id: Option<String>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTransition {
    pub id: String,
    #[sqlx(rename = "fromStepId")]
    pub from_step_id: String,
    #[sqlx(rename = "toStepId")]
    pub to_step_id: String,
    pub condition: Option<Value>,
    pub priority: i32,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowExecution {
    pub id: String,
    #[sqlx(rename = "workflowId")]
    pub workflow_id: String,
    #[sqlx(rename = "contactId")]
    pub contact_id: String,
    pub status: WorkflowExecutionStatus,
    #[sqlx(rename = "currentStepId")]
    pub current_step_id: Option<String>,
    #[sqlx(rename = "exitReason")]
    pub exit_reason: Option<String>,
    pub context: Option<Value>,
    #[sqlx(rename = "startedAt")]
    pub started_at: NaiveDateTime,
    #[sqlx(rename = "completedAt")]
    pub completed_at: Option<NaiveDateTime>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowStepExecution {
    pub id: String,
    #[sqlx(rename = "executionId")]
    pub execution_id: String,
    #[sqlx(rename = "stepId")]
    pub step_id: String,
    pub status: StepExecutionStatus,
    #[sqlx(rename = "scheduledFor")]
    pub scheduled_for: Option<NaiveDateTime>,
    #[sqlx(rename = "executeAfter")]
    pub execute_after: Option<NaiveDateTime>,
    pub output: Option<Value>,
    pub error: Option<String>,
    #[sqlx(rename = "startedAt")]
    pub started_at: Option<NaiveDateTime>,
    #[sqlx(rename = "completedAt")]
    pub completed_at: Option<NaiveDateTime>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Email {
    pub id: String,
    #[sqlx(rename = "contactId")]
    pub contact_id: String,
    #[sqlx(rename = "toName")]
    pub to_name: Option<String>,
    pub subject: String,
    pub body: String,
    pub from: String,
    #[sqlx(rename = "fromName")]
    pub from_name: Option<String>,
    #[sqlx(rename = "replyTo")]
    pub reply_to: Option<String>,
    pub headers: Option<Value>,
    pub attachments: Option<Value>,
    #[sqlx(rename = "messageId")]
    pub message_id: Option<String>,
    #[sqlx(rename = "sourceType")]
    pub source_type: EmailSourceType,
    #[sqlx(rename = "templateId")]
    pub template_id: Option<String>,
    #[sqlx(rename = "campaignId")]
    pub campaign_id: Option<String>,
    #[sqlx(rename = "workflowExecutionId")]
    pub workflow_execution_id: Option<String>,
    #[sqlx(rename = "workflowStepExecutionId")]
    pub workflow_step_execution_id: Option<String>,
    pub status: EmailStatus,
    #[sqlx(rename = "sentAt")]
    pub sent_at: Option<NaiveDateTime>,
    #[sqlx(rename = "deliveredAt")]
    pub delivered_at: Option<NaiveDateTime>,
    #[sqlx(rename = "openedAt")]
    pub opened_at: Option<NaiveDateTime>,
    #[sqlx(rename = "clickedAt")]
    pub clicked_at: Option<NaiveDateTime>,
    #[sqlx(rename = "bouncedAt")]
    pub bounced_at: Option<NaiveDateTime>,
    #[sqlx(rename = "complainedAt")]
    pub complained_at: Option<NaiveDateTime>,
    pub opens: i32,
    pub clicks: i32,
    pub error: Option<String>,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[sqlx(rename = "updatedAt")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub data: Option<Value>,
    #[sqlx(rename = "projectId")]
    pub project_id: String,
    #[sqlx(rename = "contactId")]
    pub contact_id: Option<String>,
    #[sqlx(rename = "emailId")]
    pub email_id: Option<String>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    #[sqlx(rename = "statusCode")]
    pub status_code: i32,
    pub duration: i32,
    #[sqlx(rename = "projectId")]
    pub project_id: Option<String>,
    #[sqlx(rename = "userId")]
    pub user_id: Option<String>,
    #[sqlx(rename = "authType")]
    pub auth_type: Option<String>,
    pub ip: Option<String>,
    #[sqlx(rename = "userAgent")]
    pub user_agent: Option<String>,
    #[sqlx(rename = "errorCode")]
    pub error_code: Option<String>,
    #[sqlx(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[sqlx(rename = "requestSize")]
    pub request_size: Option<i32>,
    #[sqlx(rename = "responseSize")]
    pub response_size: Option<i32>,
    #[sqlx(rename = "createdAt")]
    pub created_at: NaiveDateTime,
}
