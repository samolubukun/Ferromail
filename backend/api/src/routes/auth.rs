use axum::{
    routing::post,
    Router,
    Json,
    http::StatusCode,
    Extension,
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub error: Option<String>,
    pub user_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

pub fn router() -> Router {
    Router::new()
        .route("/signup", post(signup_handler))
        .route("/login", post(login_handler))
}

async fn signup_handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<AuthRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    if payload.email.trim().is_empty() || payload.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                success: false,
                token: None,
                error: Some("Email and password are required".to_string()),
                user_id: None,
                project_id: None,
            }),
        );
    }

    // Check if user already exists
    match db::find_user_by_email(&pool, &payload.email).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some("User with this email already exists".to_string()),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some(format!("Database lookup error: {:?}", e)),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
        _ => {}
    }

    // Hash password
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some("Failed to hash password".to_string()),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
    };

    let user_id = Uuid::new_v4().to_string();
    let project_id = Uuid::new_v4().to_string();

    // Create User & Default Project inside db queries
    let user = match db::create_user(&pool, &user_id, &payload.email, &password_hash).await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some(format!("Failed to register user: {:?}", e)),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
    };

    // Create default project
    if let Err(e) = db::create_project(&pool, &project_id, "My First Project", &user.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                success: false,
                token: None,
                error: Some(format!("User registered, but failed to create default project: {:?}", e)),
                user_id: Some(user.id),
                project_id: None,
            }),
        );
    }

    // Generate JWT token
    let token = match generate_token(&user.id) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some("User registered, but failed to generate session token".to_string()),
                    user_id: Some(user.id),
                    project_id: Some(project_id),
                }),
            );
        }
    };

    (
        StatusCode::CREATED,
        Json(AuthResponse {
            success: true,
            token: Some(token),
            error: None,
            user_id: Some(user.id),
            project_id: Some(project_id),
        }),
    )
}

async fn login_handler(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<AuthRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    let user = match db::find_user_by_email(&pool, &payload.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some("Invalid email or password".to_string()),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some(format!("Database lookup error: {:?}", e)),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
    };

    // Verify password
    let password_hash = user.password.clone().unwrap_or_default();
    match verify(&payload.password, &password_hash) {
        Ok(true) => {}
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some("Invalid email or password".to_string()),
                    user_id: None,
                    project_id: None,
                }),
            );
        }
    }

    // Retrieve user's first project ID
    let projects = match db::find_projects_by_user(&pool, &user.id).await {
        Ok(p) => p,
        Err(_) => Vec::new(),
    };

    let project_id = projects.first().map(|p| p.id.clone());

    // Generate JWT token
    let token = match generate_token(&user.id) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    token: None,
                    error: Some("Failed to generate session token".to_string()),
                    user_id: Some(user.id),
                    project_id: None,
                }),
            );
        }
    };

    (
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            token: Some(token),
            error: None,
            user_id: Some(user.id),
            project_id,
        }),
    )
}

fn generate_token(user_id: &str) -> anyhow::Result<String> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret-key".to_string());
    let expiration = Utc::now() + Duration::days(7);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}
