// Feed handlers
use crate::{
    database::{feed_db::FeedDB, user_db::UserDB},
    middleware::auth::UserContext,
    state::AppState,
    AppError,
    AppResponse,
    HttpResponse,
};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

// ============================================================================
// Response & Data Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeedListResponse {
    pub feeds: Vec<FeedData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeedData {
    pub id: Uuid,
    pub author: String,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub like_count: i32,
    pub comment_count: i32,
    pub is_liked_by_user: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeedRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeedRequest {
    pub content: String,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Get global feed (public feed for all users)
/// GET /api/feed
pub async fn get_global_feed(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<HttpResponse<FeedListResponse>, AppError> {
    let feeds = FeedDB::get_feed(&state.db, &user_ctx.user_id, 50)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch global feed: {}", e)))?;

    Ok(HttpResponse::ok(FeedListResponse { feeds }))
}

/// Get user-specific feed (feeds by a specific user)
/// GET /api/feed/user/:user_id
pub async fn get_user_feed(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
) -> Result<HttpResponse<FeedListResponse>, AppError> {
    let feeds = FeedDB::get_user_feeds(&state.db, &user_id, &user_ctx.user_id, 50)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user feed: {}", e)))?;

    Ok(HttpResponse::ok(FeedListResponse { feeds }))
}

/// Create a new feed post
/// POST /api/feed
pub async fn create_feed(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<CreateFeedRequest>,
) -> Result<HttpResponse<FeedData>, AppError> {
    log::info!("Creating feed for user: {}", user_ctx.user_id);

    if req.content.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Feed content cannot be empty".to_string(),
            None,
        ));
    }

    if req.content.len() > 5000 {
        return Err(AppError::ValidationError(
            "Feed content exceeds maximum length of 5000 characters".to_string(),
            None,
        ));
    }

    let user = UserDB::get_user(&state.db, &user_ctx.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user: {}", e)))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let feed_id = Uuid::now_v7();
    let now = Utc::now().timestamp_millis();

    let feed_data = FeedData {
        id: feed_id,
        author: user_ctx.user_id.clone(),
        author_name: user.display_name,
        author_avatar_url: user.avatar_url,
        content: req.content,
        created_at: now,
        updated_at: now,
        like_count: 0,
        comment_count: 0,
        is_liked_by_user: false,
    };

    FeedDB::create_feed(&state.db, &feed_data, &user_ctx.user_id)
        .await
        .map_err(AppError::DatabaseError)?;

    log::info!("Feed created successfully: {}", feed_id);

    Ok(HttpResponse::new(
        StatusCode::CREATED,
        AppResponse {
            success: true,
            data: Some(feed_data),
            message: Some("Feed created successfully".to_string()),
            error: None,
        },
    ))
}

/// Update an existing feed post
/// PUT /api/feed/:feed_id
pub async fn update_feed(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<Uuid>,
    Json(req): Json<UpdateFeedRequest>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!("Updating feed: {} by user: {}", feed_id, user_ctx.user_id);

    if req.content.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Feed content cannot be empty".to_string(),
            None,
        ));
    }

    if req.content.len() > 5000 {
        return Err(AppError::ValidationError(
            "Feed content exceeds maximum length of 5000 characters".to_string(),
            None,
        ));
    }

    let feed = FeedDB::get_feed_by_id(&state.db, feed_id, &user_ctx.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch feed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Feed not found: {}", feed_id)))?;

    if feed.author != user_ctx.user_id {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    let now = Utc::now().timestamp_millis();

    FeedDB::update_feed(&state.db, feed_id, &user_ctx.user_id, &req.content, now)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update feed: {}", e)))?;

    log::info!("Feed updated successfully: {}", feed_id);

    Ok(HttpResponse::ok_message("Feed updated successfully"))
}

/// Delete a feed post
/// DELETE /api/feed/:feed_id
pub async fn delete_feed(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<Uuid>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!("Deleting feed: {} by user: {}", feed_id, user_ctx.user_id);

    let feed = FeedDB::get_feed_by_id(&state.db, feed_id, &user_ctx.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch feed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Feed not found: {}", feed_id)))?;

    if feed.author != user_ctx.user_id {
        return Err(AppError::Forbidden("You do not own this feed".to_string()));
    }

    FeedDB::delete_feed(&state.db, feed_id, &user_ctx.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete feed: {}", e)))?;

    log::info!("Feed deleted successfully: {}", feed_id);

    Ok(HttpResponse::ok_message("Feed deleted successfully"))
}

/// Get details of a specific feed
/// GET /api/feed/:feed_id
pub async fn get_feed_details(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<Uuid>,
) -> Result<HttpResponse<FeedData>, AppError> {
    let feed = FeedDB::get_feed_by_id(&state.db, feed_id, &user_ctx.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch feed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Feed not found: {}", feed_id)))?;

    Ok(HttpResponse::ok(feed))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/feed", get(get_global_feed))
        .route("/feed", post(create_feed))
        .route("/feed/user/:user_id", get(get_user_feed))
        .route("/feed/:feed_id", get(get_feed_details))
        .route("/feed/:feed_id", put(update_feed))
        .route("/feed/:feed_id", delete(delete_feed))
}
