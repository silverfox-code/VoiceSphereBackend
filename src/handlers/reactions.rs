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

// ============================================================================
// Request & Response Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionModel {
    pub feed_id: String,
    pub user_id: String,   // user who is reacting
    pub author_id: String, // author of the feed being reacted to
    pub reaction_type: i32,
    pub reacted_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddReactionRequest {
    pub feed_id: String,
    pub author_id: String, // author of the feed being reacted to
    #[serde(default = "default_reaction_type")]
    pub reaction_type: i32, // 1 = like, 2 = love, 3 = laugh, etc.
}

#[derive(Debug, Deserialize)]
pub struct RemoveReactionRequest {
    pub feed_id: String,
    pub author_id: String,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub feed_id: String,
    pub user_id: String,
    pub reaction_type: i32,
    pub reacted_at: i64,
}

fn default_reaction_type() -> i32 {
    1 // Default to "like"
}

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
        user_ctx.user_id,
        req.feed_id,
        req.author_id
    );

    // Validate reaction type
    if req.reaction_type < 1 || req.reaction_type > 10 {
        return Err(AppError::ValidationError(
            "Invalid reaction type. Must be between 1 and 10".to_string(),
            None,
        ));
    }

    // Prevent self-reactions (optional business logic)
    if user_ctx.user_id == req.author_id {
        log::warn!(
            "User {} attempted to react to their own feed {}",
            user_ctx.user_id,
            req.feed_id
        );
        // You can either allow or disallow self-reactions
        // For now, we'll allow it but log a warning
    }

    let now = Utc::now().timestamp();

    let reaction_data = ReactionModel {
        feed_id: req.feed_id.clone(),
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        reaction_type: req.reaction_type,
        reacted_at: now,
    };

    // Add reaction to database
    // This will:
    // 1. Insert into reactions table (with IF NOT EXISTS for idempotency)
    // 2. Increment reaction_count in feeds table
    // 3. Increment reaction_count in global_feed table
    // 4. Increment likes_given for the user
    // 5. Increment likes_received for the feed author
    ReactionDB::add_reaction(&state.db, reaction_data.clone())
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to add reaction: {}", e)))?;

    log::info!(
        "Reaction added successfully: user {} -> feed {}",
        user_ctx.user_id,
        req.feed_id
    );

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
        user_ctx.user_id,
        feed_id,
        req.author_id
    );

    // Verify the feed_id in path matches the one in body
    if feed_id != req.feed_id {
        return Err(AppError::ValidationError(
            "Feed ID in path does not match feed ID in request body".to_string(),
            None,
        ));
    }

    let reaction_data = ReactionModel {
        feed_id: req.feed_id.clone(),
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        reaction_type: 0, // Not used for removal
        reacted_at: 0,    // Not used for removal
    };

    // Remove reaction from database
    // This will:
    // 1. Delete from reactions table
    // 2. Decrement reaction_count in feeds table
    // 3. Decrement reaction_count in global_feed table
    // 4. Decrement likes_given for the user
    // 5. Decrement likes_received for the feed author
    ReactionDB::remove_reaction(&state.db, reaction_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to remove reaction: {}", e)))?;

    log::info!(
        "Reaction removed successfully: user {} -> feed {}",
        user_ctx.user_id,
        req.feed_id
    );

    Ok(HttpResponse::ok_message("Reaction removed successfully"))
}

/// Toggle a reaction (add if not exists, remove if exists)
/// POST /api/reactions/toggle
pub async fn toggle_reaction(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<AddReactionRequest>,
) -> Result<HttpResponse<serde_json::Value>, AppError> {
    log::info!(
        "User {} toggling reaction on feed {} (author: {})",
        user_ctx.user_id,
        req.feed_id,
        req.author_id
    );

    // TODO: Implement toggle logic
    // 1. Check if reaction exists
    // 2. If exists, remove it
    // 3. If not exists, add it
    // 4. Return the action taken (added or removed)

    // For now, returning a placeholder
    Err(AppError::InternalError(
        "Toggle reaction not yet implemented".to_string(),
    ))
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
