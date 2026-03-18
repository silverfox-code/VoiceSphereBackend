// Feed handlers
use crate::{
    database::feed_db::FeedDB,
    middleware::auth::UserContext,
    state::AppState, 
    AppError, 
    AppResponse, 
    HttpResponse
};
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Router,
};
use scylla::{DeserializeRow, DeserializeValue, SerializeRow, SerializeValue, serialize::row::SerializeRow};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

// ============================================================================
// Response & Data Structures
// ============================================================================

#[derive(Debug, SerializeRow, DeserializeRow,SerializeValue, DeserializeValue,Serialize, Deserialize,  Clone)]
pub struct FeedListResponse {
    pub feeds: Vec<FeedData>,
}

#[derive(Debug, SerializeRow, DeserializeRow,Serialize, Deserialize,SerializeValue, DeserializeValue,Clone)]
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
    pub user_id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeedRequest {
    pub feed_id: String,
    pub user_id: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct FeedQueryParams {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub page: i32,
}

fn default_limit() -> i32 {
    20
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Get global feed (public feed for all users)
/// GET /api/feed
pub async fn get_global_feed(
    State(state): State<AppState>,
    // Extension(user_ctx): Extension<UserContext>,
    // Query(params): Query<FeedQueryParams>,
) -> Result<HttpResponse<FeedListResponse>, AppError> {
    // log::info!("Fetching global feed for user: {}", user_ctx.user_id);

    // let limit = params.limit.min(100).max(1); // Cap at 100, minimum 1
    
    let feeds = FeedDB::get_feed(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch global feed: {}", e)))?;

    let response = FeedListResponse {
        // total: feeds.len() as i128,
        feeds,
    };

    Ok(HttpResponse::ok(response))
}

/// Get user-specific feed (feeds by a specific user)
/// GET /api/feed/user/:user_id
pub async fn get_user_feed(
    State(state): State<AppState>,
    // Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
    // Query(params): Query<FeedQueryParams>,
) -> Result<HttpResponse<FeedListResponse>, AppError> {
    // log::info!("Fetching feed for user: {} (requested by: {})", user_id, user_ctx.user_id);

    // let limit = params.limit.min(100).max(1);
    
    let feeds = FeedDB::get_user_feeds(&state.db, &user_id, 50)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user feed: {}", e)))?;

    let response = FeedListResponse {
        // total: feeds.len(),
        feeds,
    };

    Ok(HttpResponse::ok(response))
}

/// Create a new feed post
/// POST /api/feed
pub async fn create_feed(
    State(state): State<AppState>,
    // Extension(user_ctx): Extension<UserContext>, // Commented for testing without auth
    Json(req): Json<CreateFeedRequest>,
) -> Result<HttpResponse<FeedData>, AppError> {
    log::info!("Creating feed for user: {}", req.user_id);

    // Validate content
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

    // Generate feed ID using UUID v7 (time-ordered, modern standard)
    let feed_id = Uuid::now_v7();
    let now = Utc::now().timestamp();

    // TODO: Fetch user details from database
    // For now, using placeholder values
    let author_name = "User".to_string(); // Should fetch from user table
    let author_avatar_url = None; // Should fetch from user table

    let feed_data = FeedData {
        id: feed_id,
        author: req.user_id.clone(),
        author_name,
        author_avatar_url,
        content: req.content,
        created_at: now,
        updated_at: now,
        like_count: 0,
        comment_count: 0,
        is_liked_by_user: false,
    };

    // Save to database
    FeedDB::create_feed(&state.db, &feed_data, &req.user_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create feed: {}", e)))?;

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
    // Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<String>,
    Json(req): Json<UpdateFeedRequest>,
) -> Result<HttpResponse<()>, AppError> {
    log::info!("Updating feed: {} by user: {}", feed_id, req.user_id);

    // Validate content
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

    // TODO: Implement update logic
    // 1. Verify the feed exists
    // 2. Verify the user owns the feed
    // 3. Update the feed content and updated_at timestamp
    // 4. Update in both user_feed and global_feed tables

    log::info!("Feed updated successfully: {}", feed_id);

    Ok(HttpResponse::ok_message("Feed updated successfully"))
}

/// Delete a feed post
/// DELETE /api/feed/:feed_id
pub async fn delete_feed(
    State(state): State<AppState>,
    // Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<Uuid>,
) -> Result<HttpResponse<()>, AppError> {
    // log::info!("Deleting feed: {} by user: {}", feed_id, user_ctx.user_id);

    // TODO: Verify the user owns the feed before deleting
    // For now, allowing deletion

    FeedDB::delete_feed(&state.db, feed_id)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to delete feed: {}", e)))?;

    log::info!("Feed deleted successfully: {}", feed_id);

    Ok(HttpResponse::ok_message("Feed deleted successfully"))
}

/// Get details of a specific feed
/// GET /api/feed/:feed_id
pub async fn get_feed_details(
    State(state): State<AppState>,
    // Extension(user_ctx): Extension<UserContext>,
    Path(feed_id): Path<String>,
) -> Result<HttpResponse<FeedData>, AppError> {
    // log::info!("Fetching feed details: {} for user: {}", feed_id, user_ctx.user_id);

    // TODO: Implement get single feed logic
    // For now, returning a placeholder error
    Err(AppError::NotFound(format!("Feed not found: {}", feed_id)))
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
        .route("/feed/:feed_id", post(update_feed))
        .route("/feed/:feed_id", delete(delete_feed))
}
