// Feed/Post model
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub media_url: Option<Vec<String>>,
    pub likes_count: i32,
    pub comments_count: i32,
    pub reactions_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeedRequest {
    pub content: String,
    pub media_url: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct FeedResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub media_url: Option<Vec<String>>,
    pub likes_count: i32,
    pub comments_count: i32,
    pub reactions_count: i32,
    pub created_at: i64,
}
