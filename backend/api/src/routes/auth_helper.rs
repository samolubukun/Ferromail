use axum::http::HeaderMap;
use sqlx::PgPool;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

pub async fn authenticate_project(
    headers: &HeaderMap,
    pool: &PgPool,
) -> Result<db::models::Project, String> {
    let auth_header = match headers.get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(s) => s,
            Err(_) => return Err("Invalid authorization header encoding".to_string()),
        },
        None => return Err("Missing Authorization header".to_string()),
    };

    if !auth_header.starts_with("Bearer ") {
        return Err("Authorization format must be Bearer <token>".to_string());
    }

    let token = &auth_header[7..].trim();

    // 1. Try to parse as API Secret Key first (e.g. starts with sk_prod_ or sk_test_)
    if token.starts_with("sk_") || token.contains("_secret") || token.len() > 30 && !token.contains(".") {
        match db::find_project_by_secret(pool, token).await {
            Ok(Some(project)) => {
                if project.disabled {
                    return Err("Project is disabled".to_string());
                }
                return Ok(project);
            }
            Ok(None) => return Err("Invalid API key".to_string()),
            Err(e) => return Err(format!("Database error: {:?}", e)),
        }
    }

    // 2. Otherwise, treat as Dashboard JWT session token (standard JWTs contain dots)
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret-key".to_string());
    let mut validation = Validation::default();
    validation.validate_exp = true;
    
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            let user_id = token_data.claims.sub;
            // Retrieve projects associated with this user
            match db::find_projects_by_user(pool, &user_id).await {
                Ok(projects) => {
                    if let Some(project) = projects.into_iter().next() {
                        if project.disabled {
                            return Err("Project is disabled".to_string());
                        }
                        return Ok(project);
                    }
                    Err("No projects found for this user".to_string())
                }
                Err(e) => Err(format!("Failed to query user projects: {:?}", e)),
            }
        }
        Err(_) => Err("Invalid or expired session token".to_string()),
    }
}
