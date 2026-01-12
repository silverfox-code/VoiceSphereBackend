use crate::{AppError, response::HttpResponse};
use crate::database::user_db::UserDB;
use crate::utils::authenticate::{verify_google_token, verify_refresh_token, generate_jwt_token};
use crate::{state::AppState,  User};
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub google_token: String,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse{
    pub id: String,
    pub device_id: String,
    pub display_name: String,
    pub email: String,

    // Profile
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub language: String,
    pub timezone: String,

    // Status
    pub is_active: bool,

    // Auth control
    pub session_version: i32,
    pub token: String,

    pub expires_at: i64,   
}

impl LoginResponse {
    pub fn new(user: &User, token: String, expires_at: i64) -> Self {
        Self {
            id: user.id.clone(),
            device_id: user.device_id.to_string(),
            display_name: user.display_name.clone(),
            email: user.email.clone(),
            bio: user.bio.clone(),
            avatar_url: user.avatar_url.clone(),
            language: user.language.clone(),
            timezone: user.timezone.clone(),
            is_active: user.is_active,
            session_version: user.session_version,
            token,
            expires_at,
        }
    }
}

pub async fn login( 
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<HttpResponse<LoginResponse>, AppError> {
    let token = req.google_token;
    let device_id = req.device_id;

    let claims = verify_google_token(&token, &state.google_client_id)
            .await
            .map_err(|e| {
                AppError::Unauthorized(format!("Failed to verify Google token: {}", e))
            })? ;

    log::info!("User Authenticated {:?}", claims);

    //check if user exists in DB, if not create new user
     match UserDB::user_exists(&state.db, &claims.sub).await? {
        Some(existing_user) => {
            log::info!("Existing user logged in with id: {}", claims.sub);
            let now = chrono::Utc::now();
            UserDB::update_last_login(&state.db, &claims.sub, now).await?;

            let (token, expires_at) = generate_jwt_token(&existing_user, &state.jwt_secret)?;
            return Ok(HttpResponse::ok(LoginResponse::new(&existing_user, token, expires_at)));
        }
        None => {
            log::info!("New user created with id: {}", claims.sub);
            let new_user = User::create_user_from_google_claims(claims, device_id);
            UserDB::create_user(&state.db, &new_user).await?;

            let (token, expires_at) = generate_jwt_token(&new_user, &state.jwt_secret)?;
            return Ok(HttpResponse::ok(LoginResponse::new(&new_user, token, expires_at)));
        }
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<HttpResponse<LoginResponse>, AppError> {
    let token = req.google_token;
    let device_id = req.device_id;

    log::info!("Attempting to refresh session for device: {}", device_id);

    // 1. Verify token signature and device ownership
    let claims = verify_refresh_token(&token, &device_id, &state.jwt_secret)?;

    // 2. Fetch fresh user data from DB using ID from claims
    let user = UserDB::get_user(&state.db, &claims.sub)
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // 3. Validate session version (protects against revoked sessions)
    if user.session_version != claims.session_version {
         return Err(AppError::Unauthorized("Session version mismatch".to_string()));
    }

    // 4. Ensure user is still active
    if !user.is_active {
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    // 5. Generate a fresh token pair
    let (new_token, expires_at) = generate_jwt_token(&user, &state.jwt_secret)?;

    log::info!("Session successfully refreshed for user: {}", user.id);
    Ok(HttpResponse::ok(LoginResponse::new(&user, new_token, expires_at)))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
}
