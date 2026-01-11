use crate::{AppError, response::HttpResponse};
use crate::authenticate::{GoogleClaims, generate_jwt_token};
// Authentication handlers
use crate::database::user_db::UserDB;
use crate::handlers::user;
use crate::utils::authenticate::verify_google_token;
use crate::{state::AppState,  User};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub google_token: String,
    pub uuid: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse{
    pub id: String,
    pub uuid: String,
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
    pub token: String
}

impl LoginResponse {
    pub fn new(user: &User, token: String) -> Self {
        Self {
            id: user.id.clone(),
            uuid: user.uuid.clone(),
            display_name: user.display_name.clone(),
            email: user.email.clone(),
            bio: user.bio.clone(),
            avatar_url: user.avatar_url.clone(),
            language: user.language.clone(),
            timezone: user.timezone.clone(),
            is_active: user.is_active,
            session_version: user.session_version,
            token,
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<HttpResponse<LoginResponse>, AppError> {
    let token = req.google_token;
    let uuid = req.uuid;

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

            let token = generate_jwt_token(&existing_user, &state.jwt_secret)?;
            return Ok(HttpResponse::ok(LoginResponse::new (&existing_user, token)));
        }
        None => {
            log::info!("New user created with id: {}", claims.sub);
            let new_user = User::create_user_from_google_claims(claims, uuid);
            UserDB::create_user(&state.db, &new_user).await?;

            let token = generate_jwt_token(&new_user, &state.jwt_secret)?;
            return Ok(HttpResponse::ok(LoginResponse::new(&new_user, token)));
        }
    }
}

pub async fn refresh(
    State(_state): State<AppState>,
    Json(_req): Json<LoginRequest>,
) -> impl IntoResponse {
    // TODO: Implement refresh logic
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"message": "Refresh endpoint"})),
    )
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
}
