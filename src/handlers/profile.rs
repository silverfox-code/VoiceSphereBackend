// Profile handlers
use crate::{
    database::user_db::UserDB,
    middleware::auth::UserContext,
    models::user::{ProfileData, UpdateProfileRequest, UserStatsData},
    state::AppState,
    AppError, HttpResponse,
};
use axum::{
    extract::{Extension, Json, Path, State},
    routing::{get, put},
    Router,
};

// ============================================================================
// Handler Functions
// ============================================================================

/// Get the authenticated user's own profile
/// GET /api/profile
pub async fn get_my_profile(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<HttpResponse<ProfileData>, AppError> {
    log::info!("Getting profile for user: {}", user_ctx.user_id);

    let user = UserDB::get_user(&state.db, &user_ctx.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::ok(ProfileData::from(user)))
}

/// Get any user's public profile
/// GET /api/profile/:user_id
pub async fn get_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<HttpResponse<ProfileData>, AppError> {
    let user = UserDB::get_user(&state.db, &user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch profile: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;

    Ok(HttpResponse::ok(ProfileData::from(user)))
}

/// Update the authenticated user's profile
/// PUT /api/profile
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!("Updating profile for user: {}", user_ctx.user_id);

    if req.username.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Username cannot be empty".to_string(),
            None,
        ));
    }
    if req.username.len() > 50 {
        return Err(AppError::ValidationError(
            "Username exceeds maximum length of 50 characters".to_string(),
            None,
        ));
    }

    if let Some(ref bio) = req.bio {
        if bio.len() > 500 {
            return Err(AppError::ValidationError(
                "Bio exceeds maximum length of 500 characters".to_string(),
                None,
            ));
        }
    }

    UserDB::update_user(
        &state.db,
        &user_ctx.user_id,
        &req.username,
        req.bio.as_deref(),
    )
    .await
    .map_err(|e| AppError::DatabaseError(format!("Failed to update profile: {}", e)))?;

    Ok(HttpResponse::ok_message("Profile updated successfully"))
}

/// Get a user's activity stats
/// GET /api/profile/:user_id/stats
pub async fn get_user_stats(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<HttpResponse<UserStatsData>, AppError> {
    let stats = UserDB::get_user_stats(&state.db, &user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user stats: {}", e)))?
        .unwrap_or(UserStatsData {
            user_id: user_id.clone(),
            topic_count: 0,
            call_count: 0,
            likes_given: 0,
            likes_received: 0,
            comments_given: 0,
            comments_received: 0,
            followers_count: 0,
            following_count: 0,
        });

    Ok(HttpResponse::ok(stats))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/profile", get(get_my_profile))
        .route("/profile", put(update_profile))
        .route("/profile/:user_id", get(get_profile))
        .route("/profile/:user_id/stats", get(get_user_stats))
}
