// WebSocket message types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    // Chat messages
    #[serde(rename = "chat_message")]
    ChatMessage {
        sender_id: String,
        receiver_id: String,
        content: String,
        message_id: String,
    },

    #[serde(rename = "message_read")]
    MessageRead {
        message_id: String,
        user_id: String,
    },

    // Status updates
    #[serde(rename = "user_online")]
    UserOnline { user_id: String },

    #[serde(rename = "user_offline")]
    UserOffline { user_id: String },

    #[serde(rename = "user_typing")]
    UserTyping {
        sender_id: String,
        receiver_id: String,
    },

    // Feed updates
    #[serde(rename = "feed_update")]
    FeedUpdate { feed_id: String },

    #[serde(rename = "new_reaction")]
    NewReaction {
        target_id: String,
        user_id: String,
        reaction_type: String,
    },

    // Notifications
    #[serde(rename = "notification")]
    Notification {
        title: String,
        body: String,
        action_url: Option<String>,
    },

    // Connection
    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,
}

impl WebSocketMessage {
    pub fn ping() -> Self {
        WebSocketMessage::Ping
    }

    pub fn pong() -> Self {
        WebSocketMessage::Pong
    }
}
