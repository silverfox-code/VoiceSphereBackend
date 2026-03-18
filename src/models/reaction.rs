use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReactionModel {
    pub feed_id: Uuid,
    pub user_id: String,
    pub author_id: String,
    pub reaction_type: i32,
    pub reacted_at: i64,
}
