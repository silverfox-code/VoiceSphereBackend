// Comments handlers
use crate::{
    database::comments_db::CommentsDB, middleware::auth::UserContext, models::comment::Comment,
    state::AppState, AppError, HttpResponse,
};
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Request & Response Structures
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub feed_id: String,
    pub author_id: String, // author of the feed being commented on
    pub comment: String,
    pub parent_comment_id: Option<String>,
    pub parent_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveCommentRequest {
    pub feed_id: String,
    pub author_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub feed_id: String,
    pub comment: String,
}

#[derive(Debug, Deserialize)]
pub struct GetCommentQuery {
    pub feed_id: String,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub feed_id: String,
    pub comment_id: String,
    pub user_id: String,
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

fn to_comment_response(c: Comment) -> CommentResponse {
    CommentResponse {
        feed_id: c.feed_id.to_string(),
        comment_id: c.comment_id.to_string(),
        user_id: c.user_id,
        comment: c.comment,
        commented_at: c.commented_at,
        parent_comment_id: c.parent_comment_id.map(|u| u.to_string()),
        parent_user_id: c.parent_user_id,
    }
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

    let feed_id = Uuid::parse_str(&req.feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let parent_comment_id = req
        .parent_comment_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::ValidationError("Invalid parent_comment_id".to_string(), None))?;

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

    let comment_id = Uuid::now_v7();
    let now = Utc::now().timestamp_millis();

    let comment_data = Comment {
        feed_id,
        comment_id,
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        comment: req.comment.clone(),
        commented_at: now,
        parent_comment_id,
        parent_user_id: req.parent_user_id.clone(),
    };

    CommentsDB::add_comment(&state.db, comment_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to add comment: {}", e)))?;

    let response = CommentResponse {
        feed_id: req.feed_id,
        comment_id: comment_id.to_string(),
        user_id: user_ctx.user_id,
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
        "User {} removing comment {} from feed {}",
        user_ctx.user_id,
        comment_id,
        req.feed_id
    );

    let feed_id = Uuid::parse_str(&req.feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let comment_id_uuid = Uuid::parse_str(&comment_id)
        .map_err(|_| AppError::ValidationError("Invalid comment_id".to_string(), None))?;

    let comment = CommentsDB::get_comment(&state.db, feed_id, comment_id_uuid)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch comment: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Comment not found: {}", comment_id)))?;

    if comment.user_id != user_ctx.user_id {
        return Err(AppError::Forbidden(
            "You do not own this comment".to_string(),
        ));
    }

    let comment_data = Comment {
        feed_id,
        comment_id: comment_id_uuid,
        user_id: user_ctx.user_id.clone(),
        author_id: req.author_id.clone(),
        comment: String::new(),
        commented_at: 0,
        parent_comment_id: None,
        parent_user_id: None,
    };

    CommentsDB::remove_comment(&state.db, comment_data)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to remove comment: {}", e)))?;

    Ok(HttpResponse::ok_message("Comment removed successfully"))
}

/// Get all comments for a feed
/// GET /api/comments/feed/:feed_id
pub async fn get_feed_comments(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<String>,
) -> Result<HttpResponse<CommentsListResponse>, AppError> {
    log::info!(
        "Fetching comments for feed: {} by user: {}",
        feed_id,
        user_ctx.user_id
    );

    let feed_id_uuid = Uuid::parse_str(&feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let comments = CommentsDB::get_comments(&state.db, feed_id_uuid, 100)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch comments: {}", e)))?;

    let total = comments.len();
    let comments = comments.into_iter().map(to_comment_response).collect();

    Ok(HttpResponse::ok(CommentsListResponse { comments, total }))
}

/// Get a specific comment by ID
/// GET /api/comments/:comment_id?feed_id=<feed_id>
pub async fn get_comment(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(comment_id): Path<String>,
    Query(query): Query<GetCommentQuery>,
) -> Result<HttpResponse<CommentResponse>, AppError> {
    log::info!(
        "Fetching comment: {} by user: {}",
        comment_id,
        user_ctx.user_id
    );

    let feed_id = Uuid::parse_str(&query.feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let comment_id_uuid = Uuid::parse_str(&comment_id)
        .map_err(|_| AppError::ValidationError("Invalid comment_id".to_string(), None))?;

    let comment = CommentsDB::get_comment(&state.db, feed_id, comment_id_uuid)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch comment: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Comment not found: {}", comment_id)))?;

    Ok(HttpResponse::ok(to_comment_response(comment)))
}

/// Update a comment
/// PUT /api/comments/:comment_id
pub async fn update_comment(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(comment_id): Path<String>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!("User {} updating comment {}", user_ctx.user_id, comment_id);

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

    let feed_id = Uuid::parse_str(&req.feed_id)
        .map_err(|_| AppError::ValidationError("Invalid feed_id".to_string(), None))?;

    let comment_id_uuid = Uuid::parse_str(&comment_id)
        .map_err(|_| AppError::ValidationError("Invalid comment_id".to_string(), None))?;

    let comment = CommentsDB::get_comment(&state.db, feed_id, comment_id_uuid)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch comment: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Comment not found: {}", comment_id)))?;

    if comment.user_id != user_ctx.user_id {
        return Err(AppError::Forbidden(
            "You do not own this comment".to_string(),
        ));
    }

    let now = Utc::now().timestamp_millis();

    CommentsDB::update_comment(&state.db, feed_id, comment_id_uuid, &req.comment, now)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update comment: {}", e)))?;

    Ok(HttpResponse::ok_message("Comment updated successfully"))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/comments", post(add_comment))
        .route("/comments/:comment_id", delete(remove_comment))
        .route("/comments/:comment_id", get(get_comment))
        .route("/comments/:comment_id", put(update_comment))
        .route("/comments/feed/:feed_id", get(get_feed_comments))
}
