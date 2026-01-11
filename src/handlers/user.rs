// // User handlers
// use axum::{
//     extract::{Json, Path},
//     http::StatusCode,
//     response::IntoResponse,
//     routing::{delete, get, put},
//     Router,
// };
// use crate::models::{CreateUserRequest, UpdateUserRequest};

// pub async fn get_user(Path(user_id): Path<String>) -> impl IntoResponse {
//     // TODO: Implement get user logic
//     (StatusCode::OK, Json(serde_json::json!({"message": "Get user endpoint"})))
// }

// pub async fn update_user(
//     Path(user_id): Path<String>,
//     Json(req): Json<UpdateUserRequest>,
// ) -> impl IntoResponse {
//     // TODO: Implement update user logic
//     (StatusCode::OK, Json(serde_json::json!({"message": "Update user endpoint"})))
// }

// pub async fn delete_user(Path(user_id): Path<String>) -> impl IntoResponse {
//     // TODO: Implement delete user logic
//     (StatusCode::NO_CONTENT, Json(serde_json::json!({"message": "Delete user endpoint"})))
// }

// pub fn routes() -> Router {
//     Router::new()
//         .route("/users/:user_id", get(get_user))
//         .route("/users/:user_id", put(update_user))
//         .route("/users/:user_id", delete(delete_user))
// }
