use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
