use std::env;
use std::time::Duration;
use sqlx::PgPool;
use serde_json::json;
use tracing::{info, error, warn};
use uuid::Uuid;
use chrono::Utc;

use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{MultiPart, SinglePart, Attachment};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Ferromail Background Worker Daemon...");

    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::establish_connection(&database_url).await?;
    info!("Database connection pool established.");

    // Spawn concurrent workers
    let pool_clone1 = pool.clone();
    let email_worker = tokio::spawn(async move {
        email_delivery_loop(pool_clone1).await;
    });

    let pool_clone2 = pool.clone();
    let campaign_worker = tokio::spawn(async move {
        campaign_scheduler_loop(pool_clone2).await;
    });

    let pool_clone3 = pool.clone();
    let workflow_worker = tokio::spawn(async move {
        workflow_delay_loop(pool_clone3).await;
    });

    // Wait on workers
    let _ = tokio::join!(email_worker, campaign_worker, workflow_worker);

    Ok(())
}

// ============================================
// EMAIL DELIVERY SIMULATION WORKER
// ============================================
async fn email_delivery_loop(pool: PgPool) {
    info!("Email delivery loop started.");
    loop {
        match fetch_and_process_emails(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Successfully processed {} emails in queue.", count);
                }
            }
            Err(e) => {
                error!("Error in email delivery loop: {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn fetch_and_process_emails(pool: &PgPool) -> anyhow::Result<usize> {
    // 1. Fetch pending emails
    let emails = sqlx::query_as::<_, db::models::Email>(
        "SELECT * FROM emails WHERE status = 'PENDING' LIMIT 50"
    )
    .fetch_all(pool)
    .await?;

    if emails.is_empty() {
        return Ok(0);
    }

    let count = emails.len();

    // 2. Load SMTP relay configuration details
    let smtp_host = env::var("SMTP_RELAY_HOST")
        .or_else(|_| env::var("SMTP_HOST"))
        .ok()
        .filter(|h| !h.trim().is_empty());

    let smtp_transport = if let Some(host) = smtp_host {
        let port = env::var("SMTP_RELAY_PORT")
            .or_else(|_| env::var("SMTP_PORT"))
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(587);

        info!("Configuring outbound SMTP relay: {}:{}", host, port);
        let mut transport_builder = SmtpTransport::relay(&host)?;
        transport_builder = transport_builder.port(port);

        let secure_env = env::var("SMTP_RELAY_SECURE")
            .or_else(|_| env::var("SMTP_SECURE"))
            .unwrap_or_default()
            .to_lowercase();
        
        let tls = match secure_env.as_str() {
            "true" | "yes" | "1" => {
                let tls_params = lettre::transport::smtp::client::TlsParameters::new(host.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to initialize TLS params: {:?}", e))?;
                Tls::Required(tls_params)
            }
            "opportunistic" => {
                let tls_params = lettre::transport::smtp::client::TlsParameters::new(host.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to initialize TLS params: {:?}", e))?;
                Tls::Opportunistic(tls_params)
            }
            "false" | "no" | "0" | "none" => Tls::None,
            _ => {
                if port == 465 {
                    let tls_params = lettre::transport::smtp::client::TlsParameters::new(host.clone())
                        .map_err(|e| anyhow::anyhow!("Failed to initialize TLS params: {:?}", e))?;
                    Tls::Required(tls_params)
                } else {
                    Tls::None
                }
            }
        };
        transport_builder = transport_builder.tls(tls);

        let user = env::var("SMTP_RELAY_USERNAME").or_else(|_| env::var("SMTP_USER"));
        let pass = env::var("SMTP_RELAY_PASSWORD").or_else(|_| env::var("SMTP_PASS"));
        if let (Ok(u), Ok(p)) = (user, pass) {
            transport_builder = transport_builder.credentials(Credentials::new(u, p));
        }

        Some(transport_builder.build())
    } else {
        None
    };

    let ses_client = if std::env::var("AWS_ACCESS_KEY_ID").is_ok() && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest()).load().await;
        Some(aws_sdk_sesv2::Client::new(&config))
    } else {
        None
    };

    for email in emails {
        // Fetch contact variables
        let contact = sqlx::query_as::<_, db::models::Contact>(
            "SELECT * FROM contacts WHERE id = $1 LIMIT 1"
        )
        .bind(&email.contact_id)
        .fetch_optional(pool)
        .await?;

        let recipient_email = match &contact {
            Some(c) => c.email.clone(),
            None => {
                warn!("No contact found for email id={}. Skipping.", email.id);
                continue;
            }
        };

        let variables = match &contact {
            Some(c) => c.data.clone().unwrap_or(json!({})),
            None => json!({}),
        };

        // Render body template
        let rendered_body = shared::render_template(&email.body, &variables);

        let api_url = env::var("API_URI").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let tracked_body = shared::inject_open_tracking_pixel(
            &shared::wrap_click_tracking_links(&rendered_body, &api_url, &email.id),
            &api_url,
            &email.id
        );

        // Check unsubscribe signal
        let has_unsubscribe = shared::detect_unsubscribe_signal(&rendered_body);

        let from_mailbox = if let Some(ref name) = email.from_name {
            format!("{} <{}>", name, email.from)
        } else {
            email.from.clone()
        };

        let to_mailbox = if let Some(ref name) = email.to_name {
            format!("{} <{}>", name, recipient_email)
        } else {
            recipient_email.clone()
        };

        let message_id_value = format!("{}@ferromail.com", Uuid::new_v4());

        let mut msg_builder = Message::builder()
            .from(from_mailbox.parse::<lettre::message::Mailbox>().map_err(|e| anyhow::anyhow!("Invalid from address: {:?}", e))?)
            .to(to_mailbox.parse::<lettre::message::Mailbox>().map_err(|e| anyhow::anyhow!("Invalid to address: {:?}", e))?)
            .subject(&email.subject)
            .message_id(Some(message_id_value.clone()));

        if let Some(ref reply_to) = email.reply_to {
            msg_builder = msg_builder.reply_to(reply_to.parse::<lettre::message::Mailbox>().map_err(|e| anyhow::anyhow!("Invalid reply-to address: {:?}", e))?);
        }

        // Note: lettre 0.11 requires a statically-typed Header impl per header name.
        if let Some(headers_val) = &email.headers {
            if let Some(obj) = headers_val.as_object() {
                if !obj.is_empty() {
                    warn!("Email id={} has {} custom header(s) stored in DB; custom header injection via lettre 0.11 requires per-name trait impls and is skipped for outbound relay.", email.id, obj.len());
                }
            }
        }

        let body_part = SinglePart::html(tracked_body.clone());

        let email_msg_res = if let Some(attachments_val) = &email.attachments {
            if let Some(arr) = attachments_val.as_array() {
                if !arr.is_empty() {
                    let mut multipart = MultiPart::mixed().singlepart(body_part);
                    for att in arr {
                        if let (Some(filename), Some(content_b64), Some(content_type)) = (
                            att.get("filename").and_then(|v| v.as_str()),
                            att.get("content").and_then(|v| v.as_str()),
                            att.get("contentType").and_then(|v| v.as_str()),
                        ) {
                            let data = rx_base64(content_b64).unwrap_or_default();
                            let ct = content_type.parse::<lettre::message::header::ContentType>()
                                .map_err(|e| anyhow::anyhow!("Invalid attachment content type: {:?}", e))?;
                            let attachment = Attachment::new(filename.to_string()).body(data, ct);
                            multipart = multipart.singlepart(attachment);
                        }
                    }
                    msg_builder.multipart(multipart)
                } else {
                    msg_builder.singlepart(body_part)
                }
            } else {
                msg_builder.singlepart(body_part)
            }
        } else {
            msg_builder.singlepart(body_part)
        };

        let mut tx = pool.begin().await?;

        match email_msg_res {
            Ok(email_msg) => {
                let mut send_success = false;
                let mut response_msg_id = message_id_value.clone();
                let mut error_msg = None;

                if let Some(ref transport) = smtp_transport {
                    info!("Sending outbound email via SMTP relay: id={}, subject='{}', to='{}'", email.id, email.subject, recipient_email);
                    match transport.send(&email_msg) {
                        Ok(smtp_resp) => {
                            info!("Email dispatched successfully via SMTP relay: {:?}", smtp_resp);
                            send_success = true;
                        }
                        Err(e) => {
                            error!("Failed to dispatch email via SMTP relay: {:?}", e);
                            error_msg = Some(e.to_string());
                        }
                    }
                } else if let Some(ref client) = ses_client {
                    info!("Sending outbound email via Amazon SES API: id={}, subject='{}', to='{}'", email.id, email.subject, recipient_email);
                    let raw_bytes = email_msg.formatted();
                    let raw_message = aws_sdk_sesv2::types::RawMessage::builder()
                        .data(aws_sdk_sesv2::primitives::Blob::new(raw_bytes))
                        .build()
                        .unwrap();
                    let content = aws_sdk_sesv2::types::EmailContent::builder()
                        .raw(raw_message)
                        .build();


                    match client.send_email().content(content).send().await {
                        Ok(ses_resp) => {
                            if let Some(msg_id) = ses_resp.message_id() {
                                response_msg_id = msg_id.to_string();
                            }
                            info!("Email dispatched successfully via Amazon SES API. Message ID: {}", response_msg_id);
                            send_success = true;
                        }
                        Err(e) => {
                            error!("Failed to dispatch email via Amazon SES API: {:?}", e);
                            error_msg = Some(e.to_string());
                        }
                    }
                } else {
                    info!(
                        "Simulating email dispatch (mock): id={}, subject='{}', to='{}', has_unsubscribe_link={}",
                        email.id, email.subject, recipient_email, has_unsubscribe
                    );
                    response_msg_id = format!("ses-mock-msg-{}", Uuid::new_v4());
                    send_success = true;
                }

                if send_success {
                    sqlx::query(
                        "UPDATE emails 
                         SET status = 'DELIVERED', 
                             body = $1, 
                             \"messageId\" = $2, 
                             \"sentAt\" = NOW(), 
                             \"deliveredAt\" = NOW(),
                             \"updatedAt\" = NOW() 
                         WHERE id = $3"
                    )
                    .bind(&tracked_body)
                    .bind(&response_msg_id)
                    .bind(&email.id)
                    .execute(&mut *tx)
                    .await?;

                    let event_id = Uuid::new_v4().to_string();
                    sqlx::query(
                        "INSERT INTO events (id, name, data, \"projectId\", \"contactId\", \"emailId\", \"createdAt\")
                         VALUES ($1, 'email.delivered', $2, $3, $4, $5, NOW())"
                    )
                    .bind(&event_id)
                    .bind(json!({
                        "subject": email.subject,
                        "to": recipient_email,
                        "messageId": response_msg_id,
                        "sourceType": email.source_type,
                    }))
                    .bind(&email.project_id)
                    .bind(&email.contact_id)
                    .bind(&email.id)
                    .execute(&mut *tx)
                    .await?;
                } else {
                    let err_str = error_msg.unwrap_or_else(|| "Unknown sending failure".to_string());
                    sqlx::query(
                        "UPDATE emails 
                         SET status = 'FAILED', 
                             body = $1, 
                             error = $2,
                             \"updatedAt\" = NOW() 
                         WHERE id = $3"
                    )
                    .bind(&tracked_body)
                    .bind(&err_str)
                    .bind(&email.id)
                    .execute(&mut *tx)
                    .await?;

                    let event_id = Uuid::new_v4().to_string();
                    sqlx::query(
                        "INSERT INTO events (id, name, data, \"projectId\", \"contactId\", \"emailId\", \"createdAt\")
                         VALUES ($1, 'email.failed', $2, $3, $4, $5, NOW())"
                    )
                    .bind(&event_id)
                    .bind(json!({
                        "subject": email.subject,
                        "to": recipient_email,
                        "error": err_str,
                        "sourceType": email.source_type,
                    }))
                    .bind(&email.project_id)
                    .bind(&email.contact_id)
                    .bind(&email.id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
            Err(e) => {
                error!("Failed to construct email message: {:?}", e);
                let err_str = e.to_string();

                sqlx::query(
                    "UPDATE emails 
                     SET status = 'FAILED', 
                         body = $1, 
                         error = $2,
                         \"updatedAt\" = NOW() 
                     WHERE id = $3"
                )
                .bind(&tracked_body)
                .bind(&err_str)
                .bind(&email.id)
                .execute(&mut *tx)
                .await?;

                let event_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO events (id, name, data, \"projectId\", \"contactId\", \"emailId\", \"createdAt\")
                     VALUES ($1, 'email.failed', $2, $3, $4, $5, NOW())"
                )
                .bind(&event_id)
                .bind(json!({
                    "subject": email.subject,
                    "to": recipient_email,
                    "error": err_str,
                    "sourceType": email.source_type,
                }))
                .bind(&email.project_id)
                .bind(&email.contact_id)
                .bind(&email.id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
    }


    Ok(count)
}

// ============================================
// CAMPAIGN BROADCAST SCHEDULER
// ============================================
async fn campaign_scheduler_loop(pool: PgPool) {
    info!("Campaign scheduler loop started.");
    loop {
        match fetch_and_dispatch_campaigns(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Successfully dispatched {} campaigns.", count);
                }
            }
            Err(e) => {
                error!("Error in campaign scheduler loop: {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn fetch_and_dispatch_campaigns(pool: &PgPool) -> anyhow::Result<usize> {
    // Fetch campaigns that are scheduled and ready to send
    let campaigns = sqlx::query_as::<_, db::models::Campaign>(
        "SELECT * FROM campaigns 
         WHERE (status = 'SCHEDULED' AND \"scheduledFor\" <= NOW()) 
         OR (status = 'DRAFT' AND \"scheduledFor\" IS NULL AND \"sentAt\" IS NULL AND \"totalRecipients\" = -99) 
         LIMIT 5" // -99 is a sentinel we can set for immediate sending
    )
    .fetch_all(pool)
    .await?;

    if campaigns.is_empty() {
        return Ok(0);
    }

    let count = campaigns.len();

    for campaign in campaigns {
        info!("Dispatching campaign: id={}, name='{}'", campaign.id, campaign.name);

        // Update campaign status to SENDING
        sqlx::query("UPDATE campaigns SET status = 'SENDING', \"updatedAt\" = NOW() WHERE id = $1")
            .bind(&campaign.id)
            .execute(pool)
            .await?;

        // 1. Fetch audience contacts
        let contacts: Vec<db::models::Contact> = match campaign.audience_type {
            db::models::CampaignAudienceType::All => {
                sqlx::query_as::<_, db::models::Contact>(
                    "SELECT * FROM contacts WHERE \"projectId\" = $1 AND subscribed = true"
                )
                .bind(&campaign.project_id)
                .fetch_all(pool)
                .await?
            }
            db::models::CampaignAudienceType::Segment => {
                if let Some(segment_id) = &campaign.segment_id {
                    sqlx::query_as::<_, db::models::Contact>(
                        "SELECT c.* FROM contacts c
                         JOIN segment_memberships sm ON c.id = sm.\"contactId\"
                         WHERE sm.\"segmentId\" = $1 AND sm.\"exitedAt\" IS NULL AND c.subscribed = true"
                    )
                    .bind(segment_id)
                    .fetch_all(pool)
                    .await?
                } else {
                    Vec::new()
                }
            }
            db::models::CampaignAudienceType::Filtered => {
                // Fallback to all contacts for filtered/complex conditions in mock replication
                sqlx::query_as::<_, db::models::Contact>(
                    "SELECT * FROM contacts WHERE \"projectId\" = $1 AND subscribed = true"
                )
                .bind(&campaign.project_id)
                .fetch_all(pool)
                .await?
            }
        };

        let total_recipients = contacts.len() as i32;

        let mut tx = pool.begin().await?;

        // 2. Queue Email row for each contact
        for contact in &contacts {
            let email_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO emails (
                    id, \"contactId\", subject, body, \"from\", \"fromName\", \"replyTo\", 
                    status, \"sourceType\", \"campaignId\", \"projectId\", \"createdAt\", \"updatedAt\"
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'PENDING', 'CAMPAIGN', $8, $9, NOW(), NOW())"
            )
            .bind(&email_id)
            .bind(&contact.id)
            .bind(&campaign.subject)
            .bind(&campaign.body)
            .bind(&campaign.from)
            .bind(&campaign.from_name)
            .bind(&campaign.reply_to)
            .bind(&campaign.id)
            .bind(&campaign.project_id)
            .execute(&mut *tx)
            .await?;
        }

        // 3. Finalize campaign state
        sqlx::query(
            "UPDATE campaigns 
             SET status = 'SENT', 
                 \"totalRecipients\" = $1, 
                 \"sentCount\" = $1, 
                 \"sentAt\" = NOW(),
                 \"updatedAt\" = NOW() 
             WHERE id = $2"
        )
        .bind(total_recipients)
        .bind(&campaign.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!("Campaign sent out successfully to {} recipients.", total_recipients);
    }

    Ok(count)
}

// ============================================
// VISUAL WORKFLOW DELAY TIMER PROCESSOR
// ============================================
async fn workflow_delay_loop(pool: PgPool) {
    info!("Workflow delay processor loop started.");
    loop {
        match fetch_and_execute_workflow_steps(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Successfully executed {} scheduled workflow steps.", count);
                }
            }
            Err(e) => {
                error!("Error in workflow delay processor loop: {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn fetch_and_execute_workflow_steps(pool: &PgPool) -> anyhow::Result<usize> {
    // 1. Fetch scheduled steps
    let steps = sqlx::query_as::<_, db::models::WorkflowStepExecution>(
        "SELECT * FROM workflow_step_executions 
         WHERE status = 'SCHEDULED' AND \"scheduledFor\" <= NOW() 
         LIMIT 20"
    )
    .fetch_all(pool)
    .await?;

    if steps.is_empty() {
        return Ok(0);
    }

    let count = steps.len();

    for step_exec in steps {
        info!("Executing scheduled workflow step execution: id={}", step_exec.id);

        let mut tx = pool.begin().await?;

        // Mark execution step as RUNNING
        sqlx::query(
            "UPDATE workflow_step_executions 
             SET status = 'RUNNING', \"startedAt\" = NOW(), \"updatedAt\" = NOW() 
             WHERE id = $1"
        )
        .bind(&step_exec.id)
        .execute(&mut *tx)
        .await?;

        // Retrieve full execution details
        let workflow_exec = sqlx::query_as::<_, db::models::WorkflowExecution>(
            "SELECT * FROM workflow_executions WHERE id = $1 LIMIT 1"
        )
        .bind(&step_exec.execution_id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(w_exec) = workflow_exec {
            // Retrieve step metadata
            let step = sqlx::query_as::<_, db::models::WorkflowStep>(
                "SELECT * FROM workflow_steps WHERE id = $1 LIMIT 1"
            )
            .bind(&step_exec.step_id)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(w_step) = step {
                info!("Executing step details: type={:?}, name='{}'", w_step.step_type, w_step.name);

                match w_step.step_type {
                    db::models::WorkflowStepType::SendEmail => {
                        // Enqueue transactional workflow email
                        if let Some(template_id) = &w_step.template_id {
                            let template = sqlx::query_as::<_, db::models::Template>(
                                "SELECT * FROM templates WHERE id = $1 LIMIT 1"
                            )
                            .bind(template_id)
                            .fetch_optional(&mut *tx)
                            .await?;

                            if let Some(temp) = template {
                                let email_id = Uuid::new_v4().to_string();
                                sqlx::query(
                                    "INSERT INTO emails (
                                        id, \"contactId\", subject, body, \"from\", \"fromName\", \"replyTo\", 
                                        status, \"sourceType\", \"templateId\", \"workflowExecutionId\", 
                                        \"workflowStepExecutionId\", \"projectId\", \"createdAt\", \"updatedAt\"
                                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'PENDING', 'WORKFLOW', $8, $9, $10, $11, NOW(), NOW())"
                                )
                                .bind(&email_id)
                                .bind(&w_exec.contact_id)
                                .bind(&temp.subject)
                                .bind(&temp.body)
                                .bind(&temp.from)
                                .bind(&temp.from_name)
                                .bind(&temp.reply_to)
                                .bind(&temp.id)
                                .bind(&w_exec.id)
                                .bind(&step_exec.id)
                                .bind(&temp.project_id)
                                .execute(&mut *tx)
                                .await?;
                            }
                        }
                    }
                    db::models::WorkflowStepType::Webhook => {
                        if let Some(url) = w_step.config.get("url").and_then(|u| u.as_str()) {
                            info!("Dispatching workflow outgoing webhook request to {}", url);
                            
                            let client = reqwest::Client::new();
                            let payload = json!({
                                "event": "workflow.step.webhook",
                                "workflow_id": w_step.workflow_id,
                                "step_id": w_step.id,
                                "contact_id": w_exec.contact_id,
                                "execution_id": w_exec.id,
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });

                            let url_str = url.to_string();
                            tokio::spawn(async move {
                                match client.post(&url_str).json(&payload).send().await {
                                    Ok(resp) => info!("Webhook request to {} succeeded with status {}", url_str, resp.status()),
                                    Err(e) => error!("Webhook request to {} failed: {:?}", url_str, e),
                                }
                            });
                        }
                    }
                    _ => {}
                }

                // Finalize step execution
                sqlx::query(
                    "UPDATE workflow_step_executions 
                     SET status = 'COMPLETED', \"completedAt\" = NOW(), \"updatedAt\" = NOW() 
                     WHERE id = $1"
                )
                .bind(&step_exec.id)
                .execute(&mut *tx)
                .await?;

                // Find next transition step
                let next_transition = sqlx::query_as::<_, db::models::WorkflowTransition>(
                    "SELECT * FROM workflow_transitions WHERE \"fromStepId\" = $1 ORDER BY priority ASC LIMIT 1"
                )
                .bind(&w_step.id)
                .fetch_optional(&mut *tx)
                .await?;

                if let Some(transition) = next_transition {
                    // Schedule next step execution
                    let next_step_exec_id = Uuid::new_v4().to_string();
                    let next_step = sqlx::query_as::<_, db::models::WorkflowStep>(
                        "SELECT * FROM workflow_steps WHERE id = $1 LIMIT 1"
                    )
                    .bind(&transition.to_step_id)
                    .fetch_one(&mut *tx)
                    .await?;

                    let scheduled_time = if next_step.step_type == db::models::WorkflowStepType::Delay {
                        // Extract delay configuration (default to immediate)
                        let amount = next_step.config.get("amount").and_then(|a| a.as_i64()).unwrap_or(0);
                        let unit = next_step.config.get("unit").and_then(|u| u.as_str()).unwrap_or("seconds");

                        let secs = match unit {
                            "minutes" => amount * 60,
                            "hours" => amount * 3600,
                            "days" => amount * 86400,
                            _ => amount, // default seconds
                        };
                        Utc::now() + chrono::Duration::seconds(secs)
                    } else {
                        Utc::now()
                    };

                    sqlx::query(
                        "INSERT INTO workflow_step_executions (
                            id, \"executionId\", \"stepId\", status, \"scheduledFor\", \"createdAt\", \"updatedAt\"
                         ) VALUES ($1, $2, $3, 'SCHEDULED', $4, NOW(), NOW())"
                    )
                    .bind(&next_step_exec_id)
                    .bind(&w_exec.id)
                    .bind(&transition.to_step_id)
                    .bind(scheduled_time)
                    .execute(&mut *tx)
                    .await?;

                    // Update workflow execution current step
                    sqlx::query(
                        "UPDATE workflow_executions 
                         SET \"currentStepId\" = $1, \"updatedAt\" = NOW() 
                         WHERE id = $2"
                    )
                    .bind(&transition.to_step_id)
                    .bind(&w_exec.id)
                    .execute(&mut *tx)
                    .await?;
                } else {
                    // No next step, complete the workflow execution
                    sqlx::query(
                        "UPDATE workflow_executions 
                         SET status = 'COMPLETED', \"completedAt\" = NOW(), \"updatedAt\" = NOW() 
                         WHERE id = $1"
                    )
                    .bind(&w_exec.id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        tx.commit().await?;
    }

    Ok(count)
}

fn rx_base64(input: &str) -> Option<Vec<u8>> {
    let b64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buffer = Vec::new();
    let mut chunk = 0u32;
    let mut bits = 0;

    for c in input.chars() {
        if c.is_whitespace() || c == '=' {
            continue;
        }
        if let Some(val) = b64_chars.find(c) {
            chunk = (chunk << 6) | (val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                buffer.push(((chunk >> bits) & 0xFF) as u8);
            }
        }
    }
    Some(buffer)
}
