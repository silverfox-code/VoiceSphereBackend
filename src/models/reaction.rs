// Reaction model
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: String,
    pub user_id: String,
    pub target_type: String, // "feed" or "comment"
    pub target_id: String,
    pub reaction_type: String, // emoji or reaction type
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddReactionRequest {
    pub target_type: String,
    pub target_id: String,
    pub reaction_type: String,
}
