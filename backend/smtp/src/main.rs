use std::env;
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use mailin_embedded::{Server, SslConfig, Handler, Response, response};
use mail_parser::{MessageParser, Address, MimeHeaders};
use serde_json::Value;
use sqlx::PgPool;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone)]
struct SmtpConfig {
    api_uri: String,
    smtp_domain: String,
    max_recipients: usize,
    max_attachment_size_mb: usize,
    port_secure: u16,
    port_submission: u16,
    cert_path: String,
    acme_json_path: String,
}

#[derive(Debug, Default, Clone)]
struct SessionState {
    project_secret: Option<String>,
    project_id: Option<String>,
    sender_address: Option<String>,
    recipients: Vec<String>,
    raw_body: Vec<u8>,
}

#[derive(Clone)]
struct SmtpHandlerImpl {
    config: SmtpConfig,
    db_pool: PgPool,
    http_client: reqwest::Client,
    state: SessionState,
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Handle::current().block_on(future)
}

impl SmtpHandlerImpl {
    fn authenticate(&mut self, username: &str, password: &str) -> Response {
        if username.trim().is_empty() {
            return Response::custom(535, "Username must not be empty".to_string());
        }

        // Validate API key (project secret) in DB — the password IS the API key
        let project = block_on(db::find_project_by_secret(&self.db_pool, password));
        match project {
            Ok(Some(proj)) => {
                if proj.disabled {
                    return Response::custom(554, "Project is disabled".to_string());
                }
                self.state.project_secret = Some(password.to_string());
                self.state.project_id = Some(proj.id);
                response::AUTH_OK
            }
            Ok(None) => response::INVALID_CREDENTIALS,
            Err(e) => {
                error!("Database error during auth: {:?}", e);
                response::INTERNAL_ERROR
            }
        }
    }
}

impl Handler for SmtpHandlerImpl {
    fn helo(&mut self, _ip: IpAddr, _domain: &str) -> Response {
        response::OK
    }

    fn auth_plain(&mut self, _authorization_id: &str, authentication_id: &str, password: &str) -> Response {
        self.authenticate(authentication_id, password)
    }

    fn auth_login(&mut self, username: &str, password: &str) -> Response {
        self.authenticate(username, password)
    }

    fn mail(&mut self, _ip: IpAddr, _domain: &str, from: &str) -> Response {
        let project_id = match &self.state.project_id {
            Some(id) => id,
            None => return response::AUTHENTICATION_REQUIRED,
        };

        // Extract sender domain
        let sender_domain = match from.split('@').nth(1) {
            Some(dom) => dom,
            None => return Response::custom(501, "Invalid sender email format".to_string()),
        };

        // Check if domain is verified
        let is_verified = block_on(db::is_domain_verified(&self.db_pool, project_id, sender_domain));
        match is_verified {
            Ok(true) => {
                self.state.sender_address = Some(from.to_string());
                Response::custom(250, "Sender accepted".to_string())
            }
            Ok(false) => Response::custom(550, "Sender domain is not verified or not associated with your account".to_string()),
            Err(e) => {
                error!("Database error checking domain verification: {:?}", e);
                response::INTERNAL_ERROR
            }
        }
    }

    fn rcpt(&mut self, to: &str) -> Response {
        if self.state.project_id.is_none() {
            return response::AUTHENTICATION_REQUIRED;
        }

        if self.state.recipients.len() >= self.config.max_recipients {
            return Response::custom(452, format!("Maximum {} recipients allowed", self.config.max_recipients));
        }

        self.state.recipients.push(to.to_string());
        Response::custom(250, "Recipient accepted".to_string())
    }

    fn data_start(
        &mut self,
        _domain: &str,
        _from: &str,
        _is8bit: bool,
        _to: &[String],
    ) -> Response {
        response::START_DATA
    }

    fn data(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.state.raw_body.extend_from_slice(buf);
        Ok(())
    }

    fn data_end(&mut self) -> Response {
        let project_secret = match &self.state.project_secret {
            Some(secret) => secret,
            None => return response::AUTHENTICATION_REQUIRED,
        };

        if self.state.sender_address.is_none() {
            return Response::custom(503, "Bad sequence of commands (no sender)".to_string());
        }

        if self.state.recipients.is_empty() {
            return Response::custom(503, "Bad sequence of commands (no recipients)".to_string());
        }

        // Parse email using mail-parser
        let parser = MessageParser::new();
        let message = match parser.parse(&self.state.raw_body) {
            Some(msg) => msg,
            None => return Response::custom(554, "Failed to parse email message".to_string()),
        };

        // Extract subject
        let subject = match message.subject() {
            Some(subj) => subj.to_string(),
            None => return Response::custom(554, "Email must have a subject".to_string()),
        };

        if subject.trim().is_empty() {
            return Response::custom(554, "Email must have a non-empty subject".to_string());
        }

        if subject.len() > 998 {
            return Response::custom(554, "Subject line is too long (max 998 characters)".to_string());
        }

        // Extract body HTML/text
        let body_content = match message.body_html(0) {
            Some(html) => html.to_string(),
            None => match message.body_text(0) {
                Some(txt) => txt.to_string(),
                None => return Response::custom(554, "Email must have a body (HTML or text content)".to_string()),
            },
        };

        if body_content.trim().is_empty() {
            return Response::custom(554, "Email must have a non-empty body".to_string());
        }

        // Extract custom headers
        let mut custom_headers = serde_json::Map::new();
        let standard_headers = [
            "from", "to", "cc", "bcc", "subject", "date", "message-id", "mime-version",
            "content-type", "content-transfer-encoding", "reply-to", "return-path", "received", "dkim-signature"
        ];
        
        for header in message.headers() {
            let name_str = header.name.as_str().to_lowercase();
            if standard_headers.contains(&name_str.as_str()) {
                continue;
            }

            // Simple validation: printable ASCII without colon
            if !name_str.chars().all(|c| c.is_ascii() && c != ':') {
                return Response::custom(554, format!("Invalid header name: {}", header.name.as_str()));
            }

            if custom_headers.len() >= 50 {
                return Response::custom(554, "Too many custom headers (max 50)".to_string());
            }

            // Extract string value
            let val_str = format!("{:?}", header.value);
            if val_str.len() > 998 {
                return Response::custom(554, format!("Header value too long for {} (max 998 characters)", header.name.as_str()));
            }

            custom_headers.insert(header.name.as_str().to_string(), Value::String(val_str));
        }

        // Extract attachments
        let mut attachments = Vec::new();
        for att in message.attachments() {
            let b64_content = b64_encode(att.contents());
            
            let content_type_str = if let Some(ct) = att.content_type() {
                match &ct.c_subtype {
                    Some(sub) => format!("{}/{}", ct.c_type, sub),
                    None => ct.c_type.to_string(),
                }
            } else {
                "application/octet-stream".to_string()
            };

            attachments.push(serde_json::json!({
                "filename": att.attachment_name().unwrap_or("attachment"),
                "content": b64_content,
                "contentType": content_type_str
            }));
        }

        // Relay to API
        let api_uri_send = format!("{}/v1/send", self.config.api_uri);
        
        // Prepare names/emails mapping
        let from_addr = self.state.sender_address.clone().unwrap_or_default();
        let from_name = match message.from() {
            Some(addr) => match addr {
                Address::List(list) => {
                    list.first().and_then(|item| item.name.clone().map(|n| n.to_string()))
                }
                Address::Group(groups) => {
                    groups.first().and_then(|group| {
                        group.addresses.first().and_then(|item| item.name.clone().map(|n| n.to_string()))
                    })
                }
            },
            _ => None,
        };

        let from_payload = match from_name {
            Some(name) => serde_json::json!({ "name": name, "email": from_addr }),
            None => serde_json::json!(from_addr),
        };

        // Prepare recipient payloads
        let mut recipients_payload = Vec::new();
        for rcpt in &self.state.recipients {
            let mut name = None;
            
            // Search in To
            if let Some(addr) = message.to() {
                if let Some(n) = find_name_in_address(addr, rcpt) {
                    name = Some(n);
                }
            }

            // Search in Cc
            if name.is_none() {
                if let Some(addr) = message.cc() {
                    if let Some(n) = find_name_in_address(addr, rcpt) {
                        name = Some(n);
                    }
                }
            }

            match name {
                Some(n) => recipients_payload.push(serde_json::json!({ "name": n, "email": rcpt })),
                None => recipients_payload.push(serde_json::json!(rcpt)),
            }
        }

        let mut payload_map = serde_json::Map::new();
        payload_map.insert("from".to_string(), from_payload);
        payload_map.insert("to".to_string(), serde_json::json!(recipients_payload));
        payload_map.insert("subject".to_string(), serde_json::json!(subject));
        payload_map.insert("body".to_string(), serde_json::json!(body_content));
        
        if !custom_headers.is_empty() {
            payload_map.insert("headers".to_string(), Value::Object(custom_headers));
        }

        if !attachments.is_empty() {
            payload_map.insert("attachments".to_string(), serde_json::json!(attachments));
        }

        let payload = Value::Object(payload_map);

        let res = block_on(
            self.http_client
                .post(&api_uri_send)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", project_secret))
                .json(&payload)
                .send()
        );

        match res {
            Ok(http_res) => {
                if http_res.status().is_success() {
                    info!("Email successfully relayed to API for project ID: {:?}", self.state.project_id);
                    Response::custom(250, "OK - Email accepted for delivery".to_string())
                } else {
                    let status = http_res.status();
                    let err_text = block_on(http_res.text()).unwrap_or_default();
                    error!("API error relaying email ({}): {}", status, err_text);
                    Response::custom(554, format!("Failed to relay email: API returned {}", status))
                }
            }
            Err(e) => {
                error!("HTTP error relaying email: {:?}", e);
                Response::custom(451, "Temporary error relaying email to API".to_string())
            }
        }
    }
}

fn find_name_in_address(addr: &Address, email: &str) -> Option<String> {
    match addr {
        Address::List(list) => {
            for item in list {
                if item.address.as_deref() == Some(email) {
                    return item.name.clone().map(|n| n.to_string());
                }
            }
        }
        Address::Group(groups) => {
            for group in groups {
                for item in &group.addresses {
                    if item.address.as_deref() == Some(email) {
                        return item.name.clone().map(|n| n.to_string());
                    }
                }
            }
        }
    }
    None
}

fn b64_encode(data: &[u8]) -> String {
    let b64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { Some(data[i + 1]) } else { None };
        let b3 = if i + 2 < data.len() { Some(data[i + 2]) } else { None };

        let val1 = (b1 >> 2) as usize;
        let val2 = (((b1 & 0x03) << 4) | (b2.unwrap_or(0) >> 4)) as usize;
        
        result.push(b64_chars.chars().nth(val1).unwrap());
        result.push(b64_chars.chars().nth(val2).unwrap());

        if let Some(b2_val) = b2 {
            let val3 = (((b2_val & 0x0F) << 2) | (b3.unwrap_or(0) >> 6)) as usize;
            result.push(b64_chars.chars().nth(val3).unwrap());
            if let Some(b3_val) = b3 {
                let val4 = (b3_val & 0x3F) as usize;
                result.push(b64_chars.chars().nth(val4).unwrap());
            } else {
                result.push('=');
            }
        } else {
            result.push_str("==");
        }

        i += 3;
    }
    result
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    dotenvy::dotenv().ok();

    info!("Starting Ferromail SMTP Relay...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = db::establish_connection(&database_url).await?;
    info!("Database connection established.");

    let config = SmtpConfig {
        api_uri: env::var("API_URI").unwrap_or_else(|_| "http://localhost:3000".to_string()),
        smtp_domain: env::var("SMTP_DOMAIN").unwrap_or_default(),
        max_recipients: env::var("MAX_RECIPIENTS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5),
        max_attachment_size_mb: env::var("MAX_ATTACHMENT_SIZE_MB")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
        port_secure: env::var("PORT_SECURE")
            .unwrap_or_else(|_| "465".to_string())
            .parse()
            .unwrap_or(465),
        port_submission: env::var("PORT_SUBMISSION")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587),
        cert_path: env::var("CERT_PATH").unwrap_or_else(|_| "/certs".to_string()),
        acme_json_path: env::var("ACME_JSON_PATH").unwrap_or_else(|_| "/certs/acme.json".to_string()),
    };

    info!("API endpoint: {}", config.api_uri);
    info!("Max recipients per email: {}", config.max_recipients);
    if !config.smtp_domain.is_empty() {
        info!("SMTP domain: {}", config.smtp_domain);
    }

    let ssl_config = load_ssl_config(&config);
    let http_client = reqwest::Client::new();

    let handler = SmtpHandlerImpl {
        config: config.clone(),
        db_pool,
        http_client,
        state: SessionState::default(),
    };

    let ssl_config_sec = load_ssl_config(&config);
    let mut secure_server_started = false;

    if !matches!(ssl_config_sec, SslConfig::None) {
        let mut sec_server = Server::new(handler.clone());
        if let Err(e) = sec_server
            .with_name(&config.smtp_domain)
            .with_ssl(ssl_config_sec)
            .map_err(|e| anyhow::anyhow!("Secure server SSL error: {:?}", e))
        {
            error!("Failed to configure secure SMTP server: {:?}", e);
        } else if let Err(e) = sec_server.with_addr(format!("0.0.0.0:{}", config.port_secure)) {
            error!("Failed to bind secure SMTP server address: {:?}", e);
        } else {
            info!("Secure SMTP server listening on port {}...", config.port_secure);
            tokio::task::spawn_blocking(move || {
                if let Err(e) = sec_server.serve() {
                    error!("Secure SMTP server error: {}", e);
                }
            });
            secure_server_started = true;
        }
    }

    let mut server = Server::new(handler.clone());
    server
        .with_name(&config.smtp_domain)
        .with_ssl(ssl_config)
        .map_err(|e| anyhow::anyhow!("Submission SSL error: {:?}", e))?
        .with_addr(format!("0.0.0.0:{}", config.port_submission))
        .map_err(|e| anyhow::anyhow!("Submission address binding error: {:?}", e))?;

    info!("SMTP server listening on port {}...", config.port_submission);
    
    tokio::task::spawn_blocking(move || {
        if let Err(e) = server.serve() {
            error!("SMTP server error: {}", e);
        }
    });

    if !secure_server_started {
        warn!("⚠️ Running without TLS encryption on port 465. Setup certificates to enable TLS.");
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

fn load_ssl_config(config: &SmtpConfig) -> SslConfig {
    let key_path = Path::new(&config.cert_path).join("privkey.pem");
    let cert_path = Path::new(&config.cert_path).join("fullchain.pem");

    if key_path.exists() && cert_path.exists() {
        return SslConfig::Trusted {
            key_path: key_path.to_string_lossy().to_string(),
            cert_path: cert_path.to_string_lossy().to_string(),
            chain_path: String::new(),
        };
    }

    if let Some((key, cert)) = load_traefik_acme(&config.acme_json_path, &config.smtp_domain) {
        let temp_dir = Path::new("temp_certs");
        if !temp_dir.exists() {
            let _ = fs::create_dir_all(temp_dir);
        }
        let temp_key = temp_dir.join("privkey.pem");
        let temp_cert = temp_dir.join("fullchain.pem");
        
        let _ = fs::write(&temp_key, key);
        let _ = fs::write(&temp_cert, cert);

        return SslConfig::Trusted {
            key_path: temp_key.to_string_lossy().to_string(),
            cert_path: temp_cert.to_string_lossy().to_string(),
            chain_path: String::new(),
        };
    }

    SslConfig::None
}

fn load_traefik_acme(acme_path: &str, domain: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    if !Path::new(acme_path).exists() || domain.is_empty() {
        return None;
    }
    
    let content = fs::read_to_string(acme_path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;
    
    let certs_val = json.get("letsencrypt")?.get("Certificates")?;
    let certs_list = certs_val.as_array()?;

    for cert_item in certs_list {
        let item_domain = cert_item.get("domain")?;
        let is_match = if let Some(main_domain) = item_domain.get("main") {
            main_domain.as_str() == Some(domain)
        } else {
            item_domain.as_str() == Some(domain)
        };

        if is_match {
            let key_b64 = cert_item.get("key")?.as_str()?;
            let cert_b64 = cert_item.get("certificate")?.as_str()?;
            
            let key = rx_base64(key_b64)?;
            let cert = rx_base64(cert_b64)?;
            return Some((key, cert));
        }
    }
    None
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
