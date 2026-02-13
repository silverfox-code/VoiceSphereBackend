use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentModel {
    pub feed_id: String,
    pub comment_id: String,
    pub user_id: String,
    pub comment: String,
    pub commented_at: i64,
    pub parent_comment_id: Option<String>,  // NULL for top-level comments
    pub parent_user_id: Option<String>,     // NULL for top-level comments
}
