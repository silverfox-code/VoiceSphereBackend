// Comment model
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub feed_id: String,
    pub comment_id: String,
    pub user_id: String,                    // user who is commenting
    pub author_id: String,                  // author of the feed being commented on
    pub comment: String,
    pub commented_at: i64,
    pub parent_comment_id: Option<String>,  // NULL for top-level comments
    pub parent_user_id: Option<String>,     // NULL for top-level comments
}

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub feed_id: String,
    pub comment: String,
    pub parent_comment_id: Option<String>,  // For replies
}

#[derive(Debug, Deserialize)]
pub struct RemoveCommentRequest {
    pub feed_id: String,
    pub comment_id: String,
}
