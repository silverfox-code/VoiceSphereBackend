// // WebSocket handlers for Axum
// use axum::{
//     extract::ws::{WebSocket, WebSocketUpgrade},
//     response::IntoResponse,
// };
// use tokio::task::futures;
// use futures::{sink::SinkExt, stream::StreamExt};
// use serde_json;

// use crate::websocket::message::WebSocketMessage;

// pub struct WebSocketHandler;

// impl WebSocketHandler {
//     pub fn new() -> Self {
//         WebSocketHandler
//     }

//     pub async fn handle_socket_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
//         ws.on_upgrade(handle_socket)
//     }

//     pub async fn handle_message(&self, msg: &str) -> Result<String, String> {
//         // TODO: Parse WebSocket message
//         // - Deserialize JSON to WebSocketMessage enum
//         // - Route to appropriate handler based on message type
//         // - Return response
//         Ok(String::from("{}"))
//     }

//     pub async fn handle_chat_message(&self, sender_id: &str, receiver_id: &str, content: &str) -> Result<String, String> {
//         // TODO: Implement chat message handler
//         // - Validate sender and receiver
//         // - Store message in database
//         // - If receiver is online, send message via WebSocket
//         // - If offline, store for delivery when online
//         Ok(String::from("{}"))
//     }

//     pub async fn handle_user_typing(&self, sender_id: &str, receiver_id: &str) -> Result<String, String> {
//         // TODO: Broadcast typing indicator to receiver
//         Ok(String::from("{}"))
//     }

//     pub async fn handle_message_read(&self, message_id: &str) -> Result<String, String> {
//         // TODO: Mark message as read and notify sender
//         Ok(String::from("{}"))
//     }

//     pub async fn handle_new_reaction(&self, target_id: &str, user_id: &str, reaction_type: &str) -> Result<String, String> {
//         // TODO: Broadcast new reaction to users viewing the post
//         Ok(String::from("{}"))
//     }
// }

// // Handle WebSocket connection
// async fn handle_socket(socket: WebSocket) {
//     let (mut sender, mut receiver) = socket.split();

//     // Send initial ping
//     let _ = sender.send(axum::extract::ws::Message::Text(
//         serde_json::json!({"type": "ping"}).to_string()
//     )).await;

//     // Handle incoming messages
//     while let Some(msg) = receiver.next().await {
//         match msg {
//             Ok(axum::extract::ws::Message::Text(text)) => {
//                 // TODO: Process incoming WebSocket message
//                 // - Parse JSON
//                 // - Route to appropriate handler
//                 // - Send response
                
//                 if let Err(e) = sender.send(axum::extract::ws::Message::Text(
//                     serde_json::json!({"type": "pong"}).to_string()
//                 )).await {
//                     log::error!("Error sending message: {}", e);
//                     break;
//                 }
//             }
//             Ok(axum::extract::ws::Message::Close(_)) => {
//                 log::info!("WebSocket closed");
//                 break;
//             }
//             Err(e) => {
//                 log::error!("WebSocket error: {}", e);
//                 break;
//             }
//             _ => {}
//         }
//     }
// }
