// Comments handlers
use crate::{
    database::comments_db::CommentsDB,
    middleware::auth::UserContext,
    models::comment::Comment,
    state::AppState,
    AppError,
    HttpResponse,
};
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Request & Response Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentModel {
    pub feed_id: String,
    pub comment_id: String,
    pub user_id: String,
    pub comment: String,
    pub commented_at: i64,
    pub parent_comment_id: Option<String>, // NULL for top-level comments
    pub parent_user_id: Option<String>,    // NULL for top-level comments
}

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub feed_id: String,
    pub author_id: String, // author of the feed being commented on
    pub comment: String,
    pub parent_comment_id: Option<String>, // For replies to other comments
    pub parent_user_id: Option<String>,    // User who wrote the parent comment
}

#[derive(Debug, Deserialize)]
pub struct RemoveCommentRequest {
    pub feed_id: String,
    pub author_id: String, // author of the feed
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub feed_id: String,
    pub comment_id: String,
    pub user_id: String,
    pub user_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub comment: String,
    pub commented_at: i64,
    pub parent_comment_id: Option<String>,
    pub parent_user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommentsListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct CommentQueryParams {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub page: i32,
}

fn default_limit() -> i32 {
    50
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Add a comment to a feed
/// POST /api/comments
pub async fn add_comment(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<AddCommentRequest>,
) -> Result<HttpResponse<CommentResponse>, AppError> {
    log::info!(
        "User {} adding comment to feed {} (author: {})",
        user_ctx.user_id,
        req.feed_id,
        req.author_id
    );

    // Validate comment content
    if req.comment.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Comment cannot be empty".to_string(),
            None,
        ));
    }

    if req.comment.len() > 2000 {
        return Err(AppError::ValidationError(
            "Comment exceeds maximum length of 2000 characters".to_string(),
            None,
        ));
    }

    // Validate parent comment consistency
    if req.parent_comment_id.is_some() && req.parent_user_id.is_none() {
        return Err(AppError::ValidationError(
            "parent_user_id is required when parent_comment_id is provided".to_string(),
            None,
        ));
    }

    if req.parent_comment_id.is_none() && req.parent_user_id.is_some() {
        return Err(AppError::ValidationError(
            "parent_comment_id is required when parent_user_id is provided".to_string(),
            None,
        ));
    }

    let comment_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let comment_data = Comment {
        feed_id: req.feed_id.clone(),
        comment_id: comment_id.clone(),
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        comment: req.comment.clone(),
        commented_at: now,
        parent_comment_id: req.parent_comment_id.clone(),
        parent_user_id: req.parent_user_id.clone(),
    };

    // Add comment to database
    // This will:
    // 1. Insert into comments table
    // 2. Increment comment_count in feeds table
    // 3. Increment comment_count in global_feed table
    // 4. Increment comments_given for the user
    // 5. Increment comments_received for the feed author
    CommentsDB::add_comment(&state.db, comment_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to add comment: {}", e)))?;

    log::info!(
        "Comment added successfully: user {} -> feed {}, comment_id={}",
        user_ctx.user_id,
        req.feed_id,
        comment_id
    );

    // TODO: Fetch user details from database
    let response = CommentResponse {
        feed_id: req.feed_id,
        comment_id,
        user_id: user_ctx.user_id,
        user_name: Some("User".to_string()), // Should fetch from user table
        user_avatar_url: None,                // Should fetch from user table
        comment: req.comment,
        commented_at: now,
        parent_comment_id: req.parent_comment_id,
        parent_user_id: req.parent_user_id,
    };

    Ok(HttpResponse::new(
        StatusCode::CREATED,
        crate::AppResponse {
            success: true,
            data: Some(response),
            message: Some("Comment added successfully".to_string()),
            error: None,
        },
    ))
}

/// Remove a comment from a feed
/// DELETE /api/comments/:comment_id
pub async fn remove_comment(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(comment_id): Path<String>,
    Json(req): Json<RemoveCommentRequest>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!(
        "User {} removing comment {} from feed {} (author: {})",
        user_ctx.user_id,
        comment_id,
        req.feed_id,
        req.author_id
    );

    // TODO: Verify the user owns the comment before deleting
    // For now, allowing deletion

    let comment_data = Comment {
        feed_id: req.feed_id.clone(),
        comment_id: comment_id.clone(),
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        comment: String::new(), // Not used for removal
        commented_at: 0,        // Not used for removal
        parent_comment_id: None,
        parent_user_id: None,
    };

    // Remove comment from database
    // This will:
    // 1. Delete from comments table
    // 2. Decrement comment_count in feeds table
    // 3. Decrement comment_count in global_feed table
    // 4. Decrement comments_given for the user
    // 5. Decrement comments_received for the feed author
    CommentsDB::remove_comment(&state.db, comment_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to remove comment: {}", e)))?;

    log::info!(
        "Comment removed successfully: user {} -> feed {}, comment_id={}",
        user_ctx.user_id,
        req.feed_id,
        comment_id
    );

    Ok(HttpResponse::ok_message("Comment removed successfully"))
}

/// Get all comments for a specific feed
/// GET /api/comments/feed/:feed_id
pub async fn get_feed_comments(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<String>,
    Query(params): Query<CommentQueryParams>,
) -> Result<HttpResponse<CommentsListResponse>, AppError> {
    log::info!(
        "Fetching comments for feed: {} (requested by: {})",
        feed_id,
        user_ctx.user_id
    );

    let limit = params.limit.min(100).max(1);

    // TODO: Implement get comments from database
    // For now, returning empty list
    let comments = Vec::new();

    let response = CommentsListResponse {
        total: comments.len(),
        comments,
    };

    Ok(HttpResponse::ok(response))
}

/// Get a specific comment by ID
/// GET /api/comments/:comment_id
pub async fn get_comment(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(comment_id): Path<String>,
) -> Result<HttpResponse<CommentResponse>, AppError> {
    log::info!(
        "Fetching comment: {} (requested by: {})",
        comment_id,
        user_ctx.user_id
    );

    // TODO: Implement get single comment from database
    Err(AppError::NotFound(format!(
        "Comment not found: {}",
        comment_id
    )))
}

/// Update a comment
/// PUT /api/comments/:comment_id
pub async fn update_comment(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(comment_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!(
        "User {} updating comment {}",
        user_ctx.user_id,
        comment_id
    );

    // TODO: Implement update comment logic
    // 1. Verify the comment exists
    // 2. Verify the user owns the comment
    // 3. Update the comment content
    // Note: Cassandra doesn't support updating clustering keys,
    // so we might need to delete and re-insert

    Err(AppError::InternalError(
        "Update comment not yet implemented".to_string(),
    ))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/comments", post(add_comment))
        .route("/comments/:comment_id", delete(remove_comment))
        .route("/comments/:comment_id", get(get_comment))
        .route("/comments/:comment_id", post(update_comment))
        .route("/comments/feed/:feed_id", get(get_feed_comments))
}
