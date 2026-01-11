// mod chat_server;
// use chat_server::{ChatServer, WsMessage, WsEvent};
// use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware};
// use actix_cors::Cors;
// use actix_ws::Message;
// use serde::{Deserialize, Serialize};
// use std::sync::{Arc, Mutex};
// use std::collections::HashMap;
// use scylla::{Session, SessionBuilder, IntoTypedRows};
// use uuid::Uuid;
// use chrono::{DateTime, Utc};
// use futures_util::StreamExt;

// struct AppState {
//     session: Arc<Session>,
//     chat_server: Arc<Mutex<ChatServer>>,
// }

// #[derive(Serialize, Deserialize, Clone)]
// struct Topic {
//     id: Uuid,
//     title: String,
//     description: String,
//     author: String,
//     created_at: DateTime<Utc>,
//     comment_count: i32,
//     #[serde(default)]
//     reactions: HashMap<String, i32>,
// }

// #[derive(Deserialize)]
// struct CreateTopicRequest {
//     title: String,
//     description: String,
//     author: String,
// }

// #[derive(Deserialize)]
// struct UpdateTopicRequest {
//     title: String,
//     description: String,
// }

// #[derive(Serialize, Deserialize)]
// struct Comment {
//     id: Uuid,
//     topic_id: Uuid,
//     user_id: String,
//     content: String,
//     created_at: DateTime<Utc>,
//     #[serde(default)]
//     reactions: Option<HashMap<String, i32>>,
//     #[serde(default)]
//     user_reaction: Option<String>,
// }

// #[derive(Deserialize)]
// struct GetCommentsQuery {
//     user_id: Option<String>,
// }

// #[derive(Deserialize)]
// struct CreateCommentRequest {
//     user_id: String,
//     content: String,
// }

// #[derive(Deserialize)]
// struct AddReactionRequest {
//     user_id: String,
//     emoji: String,
// }

// #[derive(Serialize)]
// struct Reaction {
//     topic_id: Uuid,
//     user_id: String,
//     emoji: String,
//     created_at: DateTime<Utc>,
// }

// #[derive(Serialize)]
// struct HealthResponse {
//     status: String,
//     version: String,
// }

// #[derive(Deserialize)]
// struct LoginRequest {
//     email: String,
// }

// #[derive(Deserialize)]
// struct SignupRequest {
//     email: String,
//     name: String,
// }

// #[derive(Serialize)]
// struct AuthResponse {
//     user_id: String,
//     name: String,
//     email: String,
//     token: String,
// }

// async fn health_check() -> impl Responder {
//     HttpResponse::Ok().json(HealthResponse {
//         status: "ok".to_string(),
//         version: "0.1.0".to_string(),
//     })
// }

// // Helper function to get reaction counts for a topic
// async fn get_topic_reactions(session: &Session, topic_id: Uuid) -> HashMap<String, i32> {
//     let mut reactions = HashMap::new();
    
//     let query = "SELECT emoji FROM voicesphere.topic_reactions WHERE topic_id = ?";
//     if let Ok(rows) = session.query(query, (topic_id,)).await {
//         if let Some(rows) = rows.rows {
//             for row in rows.into_typed::<(String,)>() {
//                 if let Ok((emoji,)) = row {
//                     *reactions.entry(emoji).or_insert(0) += 1;
//                 }
//             }
//         }
//     }
    
//     reactions
// }

// async fn login(req: web::Json<LoginRequest>) -> impl Responder {
//     println!("Login attempt for: {}", req.email);
//     HttpResponse::Ok().json(AuthResponse {
//         user_id: "1".to_string(),
//         name: "Test User".to_string(),
//         email: req.email.clone(),
//         token: "mock-jwt-token".to_string(),
//     })
// }

// async fn signup(req: web::Json<SignupRequest>) -> impl Responder {
//     println!("Signup attempt for: {} ({})", req.email, req.name);
//     HttpResponse::Ok().json(AuthResponse {
//         user_id: "2".to_string(),
//         name: req.name.clone(),
//         email: req.email.clone(),
//         token: "mock-jwt-token".to_string(),
//     })
// }

// async fn get_topics(data: web::Data<AppState>) -> impl Responder {
//     // Limit to 50 most recent topics to prevent memory issues
//     let query = "SELECT id, title, description, author, created_at, comment_count FROM voicesphere.topics LIMIT 50";
    
//     match data.session.query(query, &[]).await {
//         Ok(rows) => {
//             let mut topics = Vec::new();
//             if let Some(rows) = rows.rows {
//                 for row in rows.into_typed::<(Uuid, String, String, String, DateTime<Utc>, Option<i32>)>() {
//                     match row {
//                         Ok((id, title, description, author, created_at, comment_count_opt)) => {
//                             // Handle NULL comment_count by defaulting to 0
//                             let comment_count = comment_count_opt.unwrap_or(0);
                            
//                             // Fetch reactions for this topic
//                             let reactions = get_topic_reactions(&data.session, id).await;
                            
//                             topics.push(Topic { 
//                                 id, 
//                                 title, 
//                                 description, 
//                                 author, 
//                                 created_at, 
//                                 comment_count,
//                                 reactions,
//                             });
//                         }
//                         Err(e) => println!("Error parsing row: {:?}", e),
//                     }
//                 }
//             }
            
//             // Sort by created_at descending (newest first) in memory
//             topics.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            
//             HttpResponse::Ok()
//                 .insert_header(("Cache-Control", "public, max-age=10")) // Cache for 10 seconds
//                 .json(topics)
//         }
//         Err(e) => {
//             println!("Error querying topics: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// async fn create_topic(
//     data: web::Data<AppState>,
//     req: web::Json<CreateTopicRequest>,
// ) -> impl Responder {
//     let id = Uuid::new_v4();
//     let created_at = Utc::now();

//     let query = "INSERT INTO voicesphere.topics (id, title, description, author, created_at, comment_count) VALUES (?, ?, ?, ?, ?, ?)";
//     match data.session.query(query, (id, &req.title, &req.description, &req.author, created_at, 0)).await {
//         Ok(_) => {
//             let topic = Topic {
//                 id,
//                 title: req.title.clone(),
//                 description: req.description.clone(),
//                 author: req.author.clone(),
//                 created_at,
//                 comment_count: 0,
//                 reactions: HashMap::new(),
//             };

//             // Broadcast NEW_TOPIC event
//             let event = WsEvent {
//                 event_type: "NEW_TOPIC".to_string(),
//                 data: serde_json::to_value(&topic).unwrap(),
//             };
//             // Broadcast to "global" topic or similar. For now, we can broadcast to a specific "feed" channel
//             // or just broadcast to all connected clients if we change how subscribe works.
//             // But currently subscribe is by topic_id.
//             // Let's assume the client subscribes to a special "feed" topic ID for feed updates.
//             // Or we can just broadcast to "feed".
//             data.chat_server.lock().unwrap().broadcast("feed", event).await;

//             HttpResponse::Ok().json(topic)
//         },
//         Err(e) => {
//             println!("Error creating topic: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// async fn create_comment(
//     data: web::Data<AppState>,
//     path: web::Path<Uuid>,
//     req: web::Json<CreateCommentRequest>,
// ) -> impl Responder {
//     let topic_id = path.into_inner();
//     let id = Uuid::new_v4();
//     let created_at = Utc::now();

//     let query = "INSERT INTO voicesphere.comments (topic_id, created_at, id, user_id, content) VALUES (?, ?, ?, ?, ?)";
//     match data.session.query(query, (topic_id, created_at, id, &req.user_id, &req.content)).await {
//         Ok(_) => {
//             let comment = Comment {
//                 id,
//                 topic_id,
//                 user_id: req.user_id.clone(),
//                 content: req.content.clone(),
//                 created_at,
//                 reactions: Some(HashMap::new()),
//                 user_reaction: None,
//             };

//             // Increment comment_count for the topic
//             let update_query = "UPDATE voicesphere.topics SET comment_count = comment_count + 1 WHERE id = ?";
//             if let Err(e) = data.session.query(update_query, (topic_id,)).await {
//                 println!("Error updating comment count: {:?}", e);
//             }

//             // Fetch updated topic to broadcast
//             let topic_query = "SELECT id, title, description, author, created_at, comment_count FROM voicesphere.topics WHERE id = ?";
//             if let Ok(rows) = data.session.query(topic_query, (topic_id,)).await {
//                 if let Some(rows) = rows.rows {
//                     for row in rows.into_typed::<(Uuid, String, String, String, DateTime<Utc>, i32)>() {
//                         if let Ok((id, title, description, author, created_at, comment_count)) = row {
//                             let reactions = get_topic_reactions(&data.session, id).await;
//                             let updated_topic = Topic { 
//                                 id, 
//                                 title, 
//                                 description, 
//                                 author, 
//                                 created_at, 
//                                 comment_count,
//                                 reactions,
//                             };
                            
//                             // Broadcast TOPIC_UPDATED to feed
//                             let topic_event = WsEvent {
//                                 event_type: "TOPIC_UPDATED".to_string(),
//                                 data: serde_json::to_value(&updated_topic).unwrap(),
//                             };
//                             data.chat_server.lock().unwrap().broadcast("feed", topic_event).await;
//                             break;
//                         }
//                     }
//                 }
//             }

//             // Broadcast NEW_COMMENT event
//             let event = WsEvent {
//                 event_type: "NEW_COMMENT".to_string(),
//                 data: serde_json::to_value(&comment).unwrap(),
//             };
//             data.chat_server.lock().unwrap().broadcast(&topic_id.to_string(), event).await;

//             HttpResponse::Ok().json(comment)
//         }
//         Err(e) => {
//             println!("Error creating comment: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// async fn get_comments(
//     data: web::Data<AppState>,
//     path: web::Path<Uuid>,
//     query: web::Query<GetCommentsQuery>,
// ) -> impl Responder {
//     let topic_id = path.into_inner();
//     let comments_query = "SELECT id, topic_id, user_id, content, created_at FROM voicesphere.comments WHERE topic_id = ?";

//     match data.session.query(comments_query, (topic_id,)).await {
//         Ok(rows) => {
//             let mut comments = Vec::new();
//             if let Some(rows) = rows.rows {
//                 for row in rows.into_typed::<(Uuid, Uuid, String, String, DateTime<Utc>)>() {
//                     match row {
//                         Ok((id, topic_id, user_id, content, created_at)) => {
//                             comments.push(Comment { 
//                                 id, 
//                                 topic_id, 
//                                 user_id, 
//                                 content, 
//                                 created_at,
//                                 reactions: Some(HashMap::new()),
//                                 user_reaction: None,
//                             });
//                         }
//                         Err(e) => println!("Error parsing comment row: {:?}", e),
//                     }
//                 }
//             }

//             // Fetch reactions for all comments in this topic
//             let reactions_query = "SELECT comment_id, user_id, emoji FROM voicesphere.comment_reactions WHERE topic_id = ?";
//             if let Ok(reaction_rows) = data.session.query(reactions_query, (topic_id,)).await {
//                 if let Some(rows) = reaction_rows.rows {
//                     for row in rows.into_typed::<(Uuid, String, String)>() {
//                         if let Ok((comment_id, r_user_id, emoji)) = row {
//                             if let Some(comment) = comments.iter_mut().find(|c| c.id == comment_id) {
//                                 // Update reaction count
//                                 let reactions = comment.reactions.as_mut().unwrap();
//                                 *reactions.entry(emoji.clone()).or_insert(0) += 1;

//                                 // Check if this is the current user's reaction
//                                 if let Some(current_user_id) = &query.user_id {
//                                     if &r_user_id == current_user_id {
//                                         comment.user_reaction = Some(emoji);
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }

//             // Sort by created_at descending (newest first)
//             comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
//             HttpResponse::Ok().json(comments)
//         }
//         Err(e) => {
//             println!("Error querying comments: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// async fn delete_topic(
//     data: web::Data<AppState>,
//     path: web::Path<Uuid>,
// ) -> impl Responder {
//     let topic_id = path.into_inner();
//     let query = "DELETE FROM voicesphere.topics WHERE id = ?";

//     match data.session.query(query, (topic_id,)).await {
//         Ok(_) => {
//             // Broadcast TOPIC_DELETED event
//             let event = WsEvent {
//                 event_type: "TOPIC_DELETED".to_string(),
//                 data: serde_json::json!({ "topic_id": topic_id }),
//             };
//             data.chat_server.lock().unwrap().broadcast("feed", event).await;

//             HttpResponse::Ok().finish()
//         }
//         Err(e) => {
//             println!("Error deleting topic: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// async fn update_topic(
//     data: web::Data<AppState>,
//     path: web::Path<Uuid>,
//     req: web::Json<UpdateTopicRequest>,
// ) -> impl Responder {
//     let topic_id = path.into_inner();
    
//     // First, get the existing topic to preserve author and created_at
//     let select_query = "SELECT author, created_at, comment_count FROM voicesphere.topics WHERE id = ?";
//     match data.session.query(select_query, (topic_id,)).await {
//         Ok(rows) => {
//             if let Some(rows) = rows.rows {
//                 for row in rows.into_typed::<(String, DateTime<Utc>, i32)>() {
//                     if let Ok((author, created_at, comment_count)) = row {
//                         // Update the topic
//                         let update_query = "UPDATE voicesphere.topics SET title = ?, description = ? WHERE id = ?";
//                         match data.session.query(update_query, (&req.title, &req.description, topic_id)).await {
//                             Ok(_) => {
//                                 let reactions = get_topic_reactions(&data.session, topic_id).await;
//                                 let topic = Topic {
//                                     id: topic_id,
//                                     title: req.title.clone(),
//                                     description: req.description.clone(),
//                                     author,
//                                     created_at,
//                                     comment_count,
//                                     reactions,
//                                 };

//                                 // Broadcast TOPIC_UPDATED event
//                                 let event = WsEvent {
//                                     event_type: "TOPIC_UPDATED".to_string(),
//                                     data: serde_json::to_value(&topic).unwrap(),
//                                 };
//                                 data.chat_server.lock().unwrap().broadcast("feed", event).await;

//                                 return HttpResponse::Ok().json(topic);
//                             }
//                             Err(e) => {
//                                 println!("Error updating topic: {:?}", e);
//                                 return HttpResponse::InternalServerError().finish();
//                             }
//                         }
//                     }
//                 }
//             }
//             HttpResponse::NotFound().finish()
//         }
//         Err(e) => {
//             println!("Error fetching topic: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// #[derive(Deserialize)]
// struct DeleteCommentRequest {
//     created_at: DateTime<Utc>,
// }

// async fn delete_comment(
//     data: web::Data<AppState>,
//     path: web::Path<(Uuid, Uuid)>, // topic_id, comment_id
//     req: web::Json<DeleteCommentRequest>,
// ) -> impl Responder {
//     let (topic_id, comment_id) = path.into_inner();
//     // We need created_at because it's part of the clustering key
//     let query = "DELETE FROM voicesphere.comments WHERE topic_id = ? AND created_at = ? AND id = ?";

//     match data.session.query(query, (topic_id, req.created_at, comment_id)).await {
//         Ok(_) => {
//             // Broadcast COMMENT_DELETED event
//             let event = WsEvent {
//                 event_type: "COMMENT_DELETED".to_string(),
//                 data: serde_json::json!({ 
//                     "topic_id": topic_id,
//                     "comment_id": comment_id 
//                 }),
//             };
//             data.chat_server.lock().unwrap().broadcast(&topic_id.to_string(), event).await;

//             HttpResponse::Ok().finish()
//         }
//         Err(e) => {
//             println!("Error deleting comment: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// // Add/toggle reaction endpoint - enforces one reaction per user
// async fn add_reaction(
//     data: web::Data<AppState>,
//     path: web::Path<Uuid>,
//     req: web::Json<AddReactionRequest>,
// ) -> impl Responder {
//     let topic_id = path.into_inner();
//     let created_at = Utc::now();

//     // First, check if user already has the SAME reaction (toggle off)
//     let check_same_query = "SELECT emoji FROM voicesphere.topic_reactions WHERE topic_id = ? AND user_id = ? AND emoji = ?";
//     let has_same_reaction = match data.session.query(check_same_query, (topic_id, &req.user_id, &req.emoji)).await {
//         Ok(rows) => rows.rows.is_some() && !rows.rows.unwrap().is_empty(),
//         Err(_) => false,
//     };

//     if has_same_reaction {
//         // Remove the reaction (toggle off)
//         let delete_query = "DELETE FROM voicesphere.topic_reactions WHERE topic_id = ? AND user_id = ? AND emoji = ?";
//         if let Err(e) = data.session.query(delete_query, (topic_id, &req.user_id, &req.emoji)).await {
//             println!("Error removing reaction: {:?}", e);
//             return HttpResponse::InternalServerError().finish();
//         }
//     } else {
//         // Remove ALL existing reactions from this user for this topic (enforce single reaction)
//         let get_existing_query = "SELECT emoji FROM voicesphere.topic_reactions WHERE topic_id = ? AND user_id = ? ALLOW FILTERING";
//         if let Ok(rows) = data.session.query(get_existing_query, (topic_id, &req.user_id)).await {
//             if let Some(rows) = rows.rows {
//                 for row in rows.into_typed::<(String,)>() {
//                     if let Ok((old_emoji,)) = row {
//                         let delete_old = "DELETE FROM voicesphere.topic_reactions WHERE topic_id = ? AND user_id = ? AND emoji = ?";
//                         let _ = data.session.query(delete_old, (topic_id, &req.user_id, &old_emoji)).await;
//                     }
//                 }
//             }
//         }
        
//         // Add new reaction
//         let insert_query = "INSERT INTO voicesphere.topic_reactions (topic_id, user_id, emoji, created_at) VALUES (?, ?, ?, ?)";
//         if let Err(e) = data.session.query(insert_query, (topic_id, &req.user_id, &req.emoji, created_at)).await {
//             println!("Error adding reaction: {:?}", e);
//             return HttpResponse::InternalServerError().finish();
//         }
//     }

//     // Fetch updated topic with reactions
//     let topic_query = "SELECT id, title, description, author, created_at, comment_count FROM voicesphere.topics WHERE id = ?";
//     match data.session.query(topic_query, (topic_id,)).await {
//         Ok(rows) => {
//             if let Some(rows) = rows.rows {
//                 for row in rows.into_typed::<(Uuid, String, String, String, DateTime<Utc>, i32)>() {
//                     if let Ok((id, title, description, author, created_at, comment_count)) = row {
//                         let reactions = get_topic_reactions(&data.session, id).await;
//                         let updated_topic = Topic {
//                             id,
//                             title,
//                             description,
//                             author,
//                             created_at,
//                             comment_count,
//                             reactions,
//                         };

//                         // Broadcast TOPIC_UPDATED to feed
//                         let event = WsEvent {
//                             event_type: "TOPIC_UPDATED".to_string(),
//                             data: serde_json::to_value(&updated_topic).unwrap(),
//                         };
//                         data.chat_server.lock().unwrap().broadcast("feed", event).await;

//                         return HttpResponse::Ok().json(updated_topic);
//                     }
//                 }
//             }
//             HttpResponse::NotFound().finish()
//         }
//         Err(e) => {
//             println!("Error fetching topic: {:?}", e);
//             HttpResponse::InternalServerError().finish()
//         }
//     }
// }

// // Add/toggle reaction on a comment - enforces one reaction per user per comment
// async fn add_comment_reaction(
//     data: web::Data<AppState>,
//     path: web::Path<(Uuid, Uuid)>,
//     req: web::Json<AddReactionRequest>,
// ) -> impl Responder {
//     let (topic_id, comment_id) = path.into_inner();
//     let created_at = Utc::now();

//     // Check if user already has a reaction
//     let check_query = "SELECT emoji FROM voicesphere.comment_reactions WHERE topic_id = ? AND comment_id = ? AND user_id = ?";
//     match data.session.query(check_query, (topic_id, comment_id, &req.user_id)).await {
//         Ok(rows) => {
//             if let Some(rows) = rows.rows {
//                 for row in rows.into_typed::<(String,)>() {
//                     if let Ok((existing_emoji,)) = row {
//                         if existing_emoji == req.emoji {
//                             // Same emoji - remove it (toggle off)
//                             let delete_query = "DELETE FROM voicesphere.comment_reactions WHERE topic_id = ? AND comment_id = ? AND user_id = ?";
//                             if let Err(e) = data.session.query(delete_query, (topic_id, comment_id, &req.user_id)).await {
//                                 println!("Error removing comment reaction: {:?}", e);
//                                 return HttpResponse::InternalServerError().finish();
//                             }
                            
//                             // Broadcast reaction removed
//                             let reaction_event = WsEvent {
//                                 event_type: "COMMENT_REACTION".to_string(),
//                                 data: serde_json::json!({
//                                     "topic_id": topic_id.to_string(),
//                                     "comment_id": comment_id.to_string(),
//                                     "user_id": &req.user_id,
//                                     "emoji": &req.emoji,
//                                     "action": "removed"
//                                 }),
//                             };
//                             data.chat_server.lock().unwrap().broadcast(&format!("topic_{}", topic_id), reaction_event).await;
                            
//                             return HttpResponse::Ok().json(serde_json::json!({"status": "removed"}));
//                         } else {
//                             // Different emoji - update it
//                             let update_query = "UPDATE voicesphere.comment_reactions SET emoji = ?, created_at = ? WHERE topic_id = ? AND comment_id = ? AND user_id = ?";
//                             if let Err(e) = data.session.query(update_query, (&req.emoji, created_at, topic_id, comment_id, &req.user_id)).await {
//                                 println!("Error updating comment reaction: {:?}", e);
//                                 return HttpResponse::InternalServerError().finish();
//                             }
                            
//                             // Broadcast comment reaction update
//                             let reaction_event = WsEvent {
//                                 event_type: "COMMENT_REACTION".to_string(),
//                                 data: serde_json::json!({
//                                     "topic_id": topic_id.to_string(),
//                                     "comment_id": comment_id.to_string(),
//                                     "user_id": &req.user_id,
//                                     "emoji": &req.emoji,
//                                     "old_emoji": existing_emoji,
//                                     "action": "updated"
//                                 }),
//                             };
//                             data.chat_server.lock().unwrap().broadcast(&format!("topic_{}", topic_id), reaction_event).await;
                            
//                             return HttpResponse::Ok().json(serde_json::json!({"status": "updated", "emoji": &req.emoji}));
//                         }
//                     }
//                 }
//             }
//         }
//         Err(e) => {
//             println!("Error checking comment reaction: {:?}", e);
//         }
//     }

//     // No existing reaction - add new one
//     let insert_query = "INSERT INTO voicesphere.comment_reactions (topic_id, comment_id, user_id, emoji, created_at) VALUES (?, ?, ?, ?, ?)";
//     if let Err(e) = data.session.query(insert_query, (topic_id, comment_id, &req.user_id, &req.emoji, created_at)).await {
//         println!("Error adding comment reaction: {:?}", e);
//         return HttpResponse::InternalServerError().finish();
//     }

//     // Broadcast new comment reaction
//     let reaction_event = WsEvent {
//         event_type: "COMMENT_REACTION".to_string(),
//         data: serde_json::json!({
//             "topic_id": topic_id.to_string(),
//             "comment_id": comment_id.to_string(),
//             "user_id": &req.user_id,
//             "emoji": &req.emoji,
//             "action": "added"
//         }),
//     };
//     data.chat_server.lock().unwrap().broadcast(&format!("topic_{}", topic_id), reaction_event).await;

//     HttpResponse::Ok().json(serde_json::json!({"status": "added", "emoji": &req.emoji}))
// }

// async fn echo_ws(
//     req: actix_web::HttpRequest, 
//     body: web::Payload,
//     data: web::Data<AppState>
// ) -> Result<HttpResponse, actix_web::Error> {
//     let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
//     let chat_server = data.chat_server.clone();
//     let session_id = Uuid::new_v4();

//     // Register session
//     chat_server.lock().unwrap().connect(session_id, session.clone());

//     actix_web::rt::spawn(async move {
//         let mut last_ping = std::time::Instant::now();
//         let ping_interval = std::time::Duration::from_secs(30);

//         loop {
//             // Send ping if needed
//             if last_ping.elapsed() >= ping_interval {
//                 if session.ping(b"").await.is_err() {
//                     break;
//                 }
//                 last_ping = std::time::Instant::now();
//             }

//             // Check for messages with timeout
//             match tokio::time::timeout(
//                 std::time::Duration::from_secs(1),
//                 msg_stream.next()
//             ).await {
//                 Ok(Some(Ok(msg))) => {
//                     match msg {
//                         Message::Text(text) => {
//                             if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
//                                 match ws_msg.action.as_str() {
//                                     "subscribe" => {
//                                         if let Some(topic_id) = ws_msg.topic_id {
//                                             chat_server.lock().unwrap().subscribe(session_id, topic_id);
//                                         }
//                                     }
//                                     _ => {}
//                                 }
//                             }
//                         }
//                         Message::Ping(bytes) => {
//                             if session.pong(&bytes).await.is_err() {
//                                 break;
//                             }
//                         }
//                         Message::Pong(_) => {
//                             // Client responded to our ping
//                         }
//                         Message::Close(_) => {
//                             chat_server.lock().unwrap().disconnect(session_id);
//                             break;
//                         }
//                         _ => {}
//                     }
//                 }
//                 Ok(Some(Err(_))) => break,
//                 Ok(None) => break,
//                 Err(_) => {
//                     // Timeout - continue loop to send ping if needed
//                     continue;
//                 }
//             }
//         }
        
//         chat_server.lock().unwrap().disconnect(session_id);
//     });

//     Ok(response)
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

//     println!("Connecting to ScyllaDB at scylla-service:9042...");
//     let session = SessionBuilder::new()
//         .known_node("scylla-service")
//         .build()
//         .await
//         .expect("Failed to connect to ScyllaDB");
    
//     let session = Arc::new(session);
//     let chat_server = Arc::new(Mutex::new(ChatServer::new()));
    
//     println!("Connected to ScyllaDB!");
//     println!("Starting server at http://0.0.0.0:8080");

//     HttpServer::new(move || {
//         let cors = Cors::permissive();

//         App::new()
//             .app_data(web::Data::new(AppState { 
//                 session: session.clone(),
//                 chat_server: chat_server.clone() 
//             }))
//             .wrap(cors)
//             .wrap(middleware::Logger::default())
//             .route("/health", web::get().to(health_check))
//             .route("/login", web::post().to(login))
//             .route("/signup", web::post().to(signup))
//             .route("/topics", web::get().to(get_topics))
//             .route("/topics", web::post().to(create_topic))
//             .route("/topics/{id}/comments", web::post().to(create_comment))
//             .route("/topics/{id}/comments", web::get().to(get_comments))
//             .route("/topics/{id}", web::put().to(update_topic))
//             .route("/topics/{id}", web::delete().to(delete_topic))
//             .route("/topics/{topic_id}/comments/{comment_id}", web::delete().to(delete_comment))
//             .route("/topics/{id}/reactions", web::post().to(add_reaction)) // New route for topic reactions
//             .route("/topics/{topic_id}/comments/{comment_id}/reactions", web::post().to(add_comment_reaction))
//             .route("/ws", web::get().to(echo_ws))
//     })
//     .bind(("0.0.0.0", 8080))?
//     .run()
//     .await
// }
