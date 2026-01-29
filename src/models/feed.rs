// Feed/Post model
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub avatar_url: Option<Vec<String>>,
    pub created_at: i64,
    pub updated_at: i64,
    pub comments: i32,
    pub reactions: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeedRequest {
    pub content: String,
}
