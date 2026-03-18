use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub feed_id: Uuid,
    pub comment_id: Uuid, // UUIDv7
    pub user_id: String,
    pub author_id: String, // author of the feed being commented on
    pub comment: String,
    pub commented_at: i64,

    pub parent_comment_id: Option<Uuid>,
    pub parent_user_id: Option<String>,
}
