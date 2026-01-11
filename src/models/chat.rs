// Chat models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub content: String,
    pub media_url: Option<String>,
    pub is_read: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user_1_id: String,
    pub user_2_id: String,
    pub last_message: Option<String>,
    pub last_message_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub receiver_id: String,
    pub content: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub receiver_id: String,
    pub content: String,
    pub media_url: Option<String>,
    pub is_read: bool,
    pub created_at: i64,
}
