use sqlx::{PgPool, postgres::PgPoolOptions};
use serde_json::Value;

pub mod models;
pub use models::*;

pub async fn establish_connection(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

// ============================================
// PROJECT & AUTH QUERIES
// ============================================

pub async fn find_project_by_secret(pool: &PgPool, secret: &str) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE secret = $1 LIMIT 1"
    )
    .bind(secret)
    .fetch_optional(pool)
    .await
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 LIMIT 1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn create_user(
    pool: &PgPool,
    id: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password, type, \"emailVerified\", \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, 'PASSWORD', false, NOW(), NOW())
         RETURNING *"
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await
}

pub async fn create_project(
    pool: &PgPool,
    id: &str,
    name: &str,
    user_id: &str,
) -> Result<Project, sqlx::Error> {
    let public_key = format!("pk_prod_{}", uuid::Uuid::new_v4().simple());
    let secret_key = format!("sk_prod_{}", uuid::Uuid::new_v4().simple());

    let mut tx = pool.begin().await?;

    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, name, public, secret, disabled, tracking, language, \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, $4, false, 'ENABLED', 'en', NOW(), NOW())
         RETURNING *"
    )
    .bind(id)
    .bind(name)
    .bind(&public_key)
    .bind(&secret_key)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO memberships (\"userId\", \"projectId\", role, \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, 'OWNER', NOW(), NOW())"
    )
    .bind(user_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(project)
}

pub async fn find_projects_by_user(pool: &PgPool, user_id: &str) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT p.* FROM projects p
         JOIN memberships m ON p.id = m.\"projectId\"
         WHERE m.\"userId\" = $1"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

// ============================================
// DOMAIN QUERIES
// ============================================

pub async fn find_domain_by_id(pool: &PgPool, id: &str) -> Result<Option<Domain>, sqlx::Error> {
    sqlx::query_as::<_, Domain>(
        "SELECT * FROM domains WHERE id = $1 LIMIT 1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn is_domain_verified(pool: &PgPool, project_id: &str, domain: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT 1 FROM domains WHERE \"projectId\" = $1 AND domain = $2 AND verified = true LIMIT 1"
    )
    .bind(project_id)
    .bind(domain)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

pub async fn create_domain(
    pool: &PgPool,
    id: &str,
    project_id: &str,
    domain: &str,
    dkim_tokens: Value,
) -> Result<Domain, sqlx::Error> {
    sqlx::query_as::<_, Domain>(
        "INSERT INTO domains (id, domain, verified, \"dkimTokens\", \"projectId\", \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, false, $3, $4, NOW(), NOW())
         RETURNING *"
    )
    .bind(id)
    .bind(domain)
    .bind(dkim_tokens)
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub async fn find_domains_by_project(pool: &PgPool, project_id: &str) -> Result<Vec<Domain>, sqlx::Error> {
    sqlx::query_as::<_, Domain>(
        "SELECT * FROM domains WHERE \"projectId\" = $1 ORDER BY \"createdAt\" DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_domain(pool: &PgPool, project_id: &str, id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "DELETE FROM domains WHERE id = $1 AND \"projectId\" = $2"
    )
    .bind(id)
    .bind(project_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

pub async fn update_domain_verified(pool: &PgPool, id: &str, verified: bool) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE domains SET verified = $1, \"updatedAt\" = NOW() WHERE id = $2"
    )
    .bind(verified)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

// ============================================
// CONTACT QUERIES
// ============================================

pub async fn create_contact(
    pool: &PgPool,
    id: &str,
    email: &str,
    project_id: &str,
    data: Option<Value>,
) -> Result<Contact, sqlx::Error> {
    sqlx::query_as::<_, Contact>(
        "INSERT INTO contacts (id, email, data, subscribed, \"projectId\", \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, true, $4, NOW(), NOW())
         ON CONFLICT (\"projectId\", email) DO UPDATE
         SET data = EXCLUDED.data, \"updatedAt\" = NOW()
         RETURNING *"
    )
    .bind(id)
    .bind(email)
    .bind(data)
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub async fn find_contacts_by_project(pool: &PgPool, project_id: &str) -> Result<Vec<Contact>, sqlx::Error> {
    sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE \"projectId\" = $1 ORDER BY \"createdAt\" DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_contact(pool: &PgPool, project_id: &str, id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "DELETE FROM contacts WHERE id = $1 AND \"projectId\" = $2"
    )
    .bind(id)
    .bind(project_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

pub async fn update_contact_subscription(
    pool: &PgPool,
    project_id: &str,
    id: &str,
    subscribed: bool,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE contacts SET subscribed = $1, \"updatedAt\" = NOW() WHERE id = $2 AND \"projectId\" = $3"
    )
    .bind(subscribed)
    .bind(id)
    .bind(project_id)
    .execute(pool)
    .await?;

    Ok(res.rows_affected() > 0)
}

// ============================================
// TEMPLATE & CAMPAIGN QUERIES
// ============================================

pub async fn create_template(
    pool: &PgPool,
    id: &str,
    name: &str,
    description: Option<&str>,
    subject: &str,
    body: &str,
    from: &str,
    from_name: Option<&str>,
    reply_to: Option<&str>,
    template_type: TemplateType,
    project_id: &str,
) -> Result<Template, sqlx::Error> {
    sqlx::query_as::<_, Template>(
        "INSERT INTO templates (id, name, description, subject, body, \"from\", \"fromName\", \"replyTo\", type, \"projectId\", \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
         RETURNING *"
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(subject)
    .bind(body)
    .bind(from)
    .bind(from_name)
    .bind(reply_to)
    .bind(template_type)
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub async fn find_templates_by_project(pool: &PgPool, project_id: &str) -> Result<Vec<Template>, sqlx::Error> {
    sqlx::query_as::<_, Template>(
        "SELECT * FROM templates WHERE \"projectId\" = $1 ORDER BY \"createdAt\" DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn create_campaign(
    pool: &PgPool,
    id: &str,
    name: &str,
    description: Option<&str>,
    subject: &str,
    body: &str,
    from: &str,
    from_name: Option<&str>,
    reply_to: Option<&str>,
    template_type: TemplateType,
    audience_type: CampaignAudienceType,
    audience_condition: Option<Value>,
    segment_id: Option<&str>,
    scheduled_for: Option<chrono::DateTime<chrono::Utc>>,
    project_id: &str,
) -> Result<Campaign, sqlx::Error> {
    sqlx::query_as::<_, Campaign>(
        "INSERT INTO campaigns (
            id, name, description, status, subject, body, \"from\", \"fromName\", \"replyTo\", type,
            \"audienceType\", \"audienceCondition\", \"segmentId\", \"scheduledFor\", \"totalRecipients\",
            \"sentCount\", \"deliveredCount\", \"openedCount\", \"clickedCount\", \"bouncedCount\", \"projectId\",
            \"createdAt\", \"updatedAt\"
         )
         VALUES (
            $1, $2, $3, 'DRAFT', $4, $5, $6, $7, $8, $9,
            $10, $11, $12, $13, 0,
            0, 0, 0, 0, 0, $14,
            NOW(), NOW()
         )
         RETURNING *"
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(subject)
    .bind(body)
    .bind(from)
    .bind(from_name)
    .bind(reply_to)
    .bind(template_type)
    .bind(audience_type)
    .bind(audience_condition)
    .bind(segment_id)
    .bind(scheduled_for)
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub async fn find_campaigns_by_project(pool: &PgPool, project_id: &str) -> Result<Vec<Campaign>, sqlx::Error> {
    sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE \"projectId\" = $1 ORDER BY \"createdAt\" DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

// ============================================
// WORKFLOW QUERIES
// ============================================

pub async fn create_workflow(
    pool: &PgPool,
    id: &str,
    name: &str,
    description: Option<&str>,
    trigger_type: WorkflowTriggerType,
    trigger_config: Option<Value>,
    allow_reentry: bool,
    project_id: &str,
) -> Result<Workflow, sqlx::Error> {
    sqlx::query_as::<_, Workflow>(
        "INSERT INTO workflows (id, name, description, enabled, \"triggerType\", \"triggerConfig\", \"allowReentry\", \"projectId\", \"createdAt\", \"updatedAt\")
         VALUES ($1, $2, $3, false, $4, $5, $6, $7, NOW(), NOW())
         RETURNING *"
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(trigger_type)
    .bind(trigger_config)
    .bind(allow_reentry)
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub async fn find_workflows_by_project(pool: &PgPool, project_id: &str) -> Result<Vec<Workflow>, sqlx::Error> {
    sqlx::query_as::<_, Workflow>(
        "SELECT * FROM workflows WHERE \"projectId\" = $1 ORDER BY \"createdAt\" DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}
