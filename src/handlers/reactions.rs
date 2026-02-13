use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactionModel {
    pub feed_id: String,
    pub user_id: String,        // user who is reacting
    pub author_id: String,      // author of the feed being reacted to
    pub reaction_type: i32,
    pub reacted_at: i64,
}

// pub async fn react_to_feed(
//     State(state): State<AppState>,
//     Json(req): Json<ReactionModel>,
// ) -> Result<HttpResponse<()>, AppError> {
//     let query = "INSERT INTO voicesphere.reactions (feed_id, user_id, reaction_type, reacted_at) VALUES (?, ?, ?, ?)";
//     state
//         .db
//         .query(
//             query,
//             (
//                 &req.feed_id,
//                 &req.user_id,
//                 req.reaction_type,
//                 req.reacted_at,
//             ),
//         )
//         .await
//         .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     Ok(HttpResponse::Ok(()))
// }
