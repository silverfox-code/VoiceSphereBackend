// // Chat handlers
// use axum::{
//     extract::{Json, Path},
//     http::StatusCode,
//     response::IntoResponse,
//     routing::{get, post, put},
//     Router,
// };
// use crate::models::SendMessageRequest;

// pub async fn get_conversations() -> impl IntoResponse {
//     // TODO: Implement get conversations logic
//     (StatusCode::OK, Json(serde_json::json!({"message": "Get conversations endpoint"})))
// }

// pub async fn get_conversation(Path(user_id): Path<String>) -> impl IntoResponse {
//     // TODO: Implement get conversation logic
//     (StatusCode::OK, Json(serde_json::json!({"message": "Get conversation endpoint"})))
// }

// pub async fn send_message(Json(req): Json<SendMessageRequest>) -> impl IntoResponse {
//     // TODO: Implement send message logic
//     (StatusCode::CREATED, Json(serde_json::json!({"message": "Send message endpoint"})))
// }

// pub async fn mark_as_read(Path(message_id): Path<String>) -> impl IntoResponse {
//     // TODO: Implement mark as read logic
//     (StatusCode::OK, Json(serde_json::json!({"message": "Mark as read endpoint"})))
// }

// pub fn routes() -> Router {
//     Router::new()
//         .route("/chat/conversations", get(get_conversations))
//         .route("/chat/conversation/:user_id", get(get_conversation))
//         .route("/chat/message", post(send_message))
//         .route("/chat/message/:message_id/read", put(mark_as_read))
// }
