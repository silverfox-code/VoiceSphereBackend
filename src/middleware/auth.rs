// Authentication middleware for Axum
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::AppError;

/// JWT Claims structure - must match the one in authenticate.rs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,           // user_id
    pub device_id: String,
    pub session_version: i32,
    pub exp: i64,
}

/// User context to be stored in request extensions
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub device_id: String,
    pub session_version: i32,
}

// for more security we can add check for session version in database , 
// version mismatch will happen if user is logged out form current session
// but this iwll be extra overhead coz on refresh token we are checking session version
// so we can skip this check for now
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".to_string()))?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());

    let claims = verify_token(token, &jwt_secret)?;

    let user_context = UserContext {
        user_id: claims.sub.clone(),
        device_id: claims.device_id.clone(),
        session_version: claims.session_version,
    };

    req.extensions_mut().insert(user_context);

    Ok(next.run(req).await)
}

fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    let claims = token_data.claims;

    // Check if token is expired (jsonwebtoken crate does this automatically,
    // but we can add additional checks here if needed)
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(AppError::Unauthorized("Token expired".to_string()));
    }

    Ok(claims)
}

// Optional: Middleware variant that also checks session version against database
// Use this if you want to invalidate tokens when user logs out from another device
// 
// pub async fn auth_middleware_with_db_check(
//     State(app_state): State<AppState>,
//     mut req: Request,
//     next: Next,
// ) -> Result<Response, AppError> {
//     // ... extract and verify token (same as above) ...
//     
//     // Fetch user from database
//     let user = app_state.db.get_user(&claims.sub).await?;
//     
//     // Check if session version matches
//     if user.session_version != claims.session_version {
//         return Err(AppError::Unauthorized("Session invalidated".to_string()));
//     }
//     
//     // ... continue with request ...
// }