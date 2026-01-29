// // Feed handlers
// use axum::{
//     extract::{Json, Path},
//     http::StatusCode,
//     response::IntoResponse,
//     routing::{delete, get, post},
//     Router,
// };
// use crate::{AppError, HttpResponse, models::feed};

// pub struct FeedResponse{
//     pub id: String,
//     pub author_id: String,
//     pub author_name: String,
//     pub content: String,
//     pub avatar_url: Option<Vec<String>>,
//     pub created_at: i64,
//     pub updated_at: i64,
//     pub comments: i32,
//     pub reactions: i32,
// }

// pub async fn get_feed() -> Result<HttpResponse<FeedResponse>, AppError> {
//     // TODO: Implement get feed logic (home feed)
//     Err(AppError::new(
//         StatusCode::NOT_IMPLEMENTED,
//         "Get feed not implemented".to_string(),
//     ))
// }

// pub async fn get_user_feed(Path(user_id): Path<String>) -> impl IntoResponse {
//     // TODO: Implement get user feed logic
//     (StatusCode::OK, Json(serde_json::json!({"message": "Get user feed endpoint"})))
// }

// pub async fn create_feed(Json(req): Json<CreateFeedRequest>) -> impl IntoResponse {
//     // TODO: Implement create feed logic
//     (StatusCode::CREATED, Json(serde_json::json!({"message": "Create feed endpoint"})))
// }

// pub async fn delete_feed(Path(feed_id): Path<String>) -> impl IntoResponse {
//     // TODO: Implement delete feed logic
//     (StatusCode::NO_CONTENT, Json(serde_json::json!({"message": "Delete feed endpoint"})))
// }

// pub fn routes() -> Router {
//     Router::new()
//         .route("/feed", get(get_feed))
//         .route("/feed", post(create_feed))
//         .route("/feed/user/:user_id", get(get_user_feed))
//         .route("/feed/:feed_id", delete(delete_feed))
// }
