// Reactions handlers
use crate::{
    database::reaction_db::ReactionDB,
    middleware::auth::UserContext,
    state::AppState,
    AppError,
    HttpResponse,
};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    routing::{delete, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Request & Response Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionModel {
    pub feed_id: Uuid,
    pub user_id: String,
    pub author_id: String,
    pub reaction_type: i32,
    pub reacted_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddReactionRequest {
    pub feed_id: String,
    pub author_id: String,
    #[serde(default = "default_reaction_type")]
    pub reaction_type: i32,
}

#[derive(Debug, Deserialize)]
pub struct RemoveReactionRequest {
    pub author_id: String,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub feed_id: String,
    pub user_id: String,
    pub reaction_type: i32,
    pub reacted_at: i64,
}

fn default_reaction_type() -> i32 { 1 }

// ============================================================================
// Handler Functions
// ============================================================================

/// Add a reaction to a feed
/// POST /api/reactions
pub async fn add_reaction(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<AddReactionRequest>,
) -> Result<HttpResponse<ReactionResponse>, AppError> {
    log::info!(
        "User {} adding reaction to feed {} (author: {})",
        user_ctx.user_id, req.feed_id, req.author_id
    );

    if req.reaction_type < 1 || req.reaction_type > 10 {
        return Err(AppError::ValidationError(
            "Invalid reaction type. Must be between 1 and 10".to_string(), None,
        ));
    }

    let feed_id = Uuid::parse_str(&req.feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let now = Utc::now().timestamp_millis();

    let reaction_data = ReactionModel {
        feed_id,
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        reaction_type: req.reaction_type,
        reacted_at: now,
    };

    ReactionDB::add_reaction(&state.db, reaction_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to add reaction: {}", e)))?;

    let response = ReactionResponse {
        feed_id: req.feed_id,
        user_id: user_ctx.user_id,
        reaction_type: req.reaction_type,
        reacted_at: now,
    };

    Ok(HttpResponse::new(
        StatusCode::CREATED,
        crate::AppResponse {
            success: true,
            data: Some(response),
            message: Some("Reaction added successfully".to_string()),
            error: None,
        },
    ))
}

/// Remove a reaction from a feed
/// DELETE /api/reactions/:feed_id
pub async fn remove_reaction(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<String>,
    Json(req): Json<RemoveReactionRequest>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!(
        "User {} removing reaction from feed {} (author: {})",
        user_ctx.user_id, feed_id, req.author_id
    );

    let feed_id_uuid = Uuid::parse_str(&feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let reaction_data = ReactionModel {
        feed_id: feed_id_uuid,
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        reaction_type: 0,
        reacted_at: 0,
    };

    ReactionDB::remove_reaction(&state.db, reaction_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to remove reaction: {}", e)))?;

    Ok(HttpResponse::ok_message("Reaction removed successfully"))
}

/// Toggle a reaction (add if not exists, remove if exists)
/// POST /api/reactions/toggle
pub async fn toggle_reaction(
    State(_state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<AddReactionRequest>,
) -> Result<HttpResponse<serde_json::Value>, AppError> {
    log::info!(
        "User {} toggling reaction on feed {} (author: {})",
        user_ctx.user_id, req.feed_id, req.author_id
    );

    // TODO: Check if reaction exists, remove if so, add if not
    Err(AppError::InternalError("Toggle reaction not yet implemented".to_string()))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/reactions", post(add_reaction))
        .route("/reactions/:feed_id", delete(remove_reaction))
        .route("/reactions/toggle", post(toggle_reaction))
}
