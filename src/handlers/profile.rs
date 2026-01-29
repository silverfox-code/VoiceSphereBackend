// Profile handlers
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use crate::middleware::auth::UserContext;

pub async fn get_profile(Path(_user_id): Path<String>) -> impl IntoResponse {
    // TODO: Implement get profile logic
    (StatusCode::OK, Json(serde_json::json!({"message": "Get profile endpoint"})))
}

pub async fn get_my_profile(
    Extension(user_ctx): Extension<UserContext>,
) -> impl IntoResponse {
    // The user_ctx is automatically injected by the auth middleware
    // It contains: user_id, device_id, session_version
    
    log::info!("Getting profile for user: {}", user_ctx.user_id);
    
    // TODO: Fetch user data from database using user_ctx.user_id
    // let user = UserDB::get_user(&state.db, &user_ctx.user_id).await?;
    
    (StatusCode::OK, Json(serde_json::json!({
        "message": "Get my profile endpoint",
        "user_id": user_ctx.user_id,
        "device_id": user_ctx.device_id,
        "session_version": user_ctx.session_version,
    })))
}

pub async fn update_profile(
    Extension(user_ctx): Extension<UserContext>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    // User can only update their own profile
    log::info!("Updating profile for user: {}", user_ctx.user_id);
    
    // TODO: Implement update profile logic
    (StatusCode::OK, Json(serde_json::json!({
        "message": "Update profile endpoint",
        "user_id": user_ctx.user_id,
    })))
}

pub async fn get_user_stats(Path(_user_id): Path<String>) -> impl IntoResponse {
    // TODO: Implement get user stats logic (followers, posts, etc.)
    (StatusCode::OK, Json(serde_json::json!({"message": "Get user stats endpoint"})))
}

pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/profile", get(get_my_profile))
        .route("/profile", put(update_profile))
        .route("/profile/:user_id", get(get_profile))
        .route("/profile/:user_id/stats", get(get_user_stats))
}
