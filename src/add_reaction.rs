// // Add/toggle reaction endpoint
// async fn add_reaction(
//     data: web::Data<AppState>,
//     path: web::Path<Uuid>,
//     req: web::Json<AddReactionRequest>,
// ) -> impl Responder {
//     let topic_id = path.into_inner();
//     let created_at = Utc::now();

//     // Check if reaction already exists
//     let check_query = "SELECT emoji FROM voicesphere.topic_reactions WHERE topic_id = ? AND user_id = ? AND emoji = ?";
//     let exists = match data.session.query(check_query, (topic_id, &req.user_id, &req.emoji)).await {
//         Ok(rows) => rows.rows.is_some() && !rows.rows.unwrap().is_empty(),
//         Err(_) => false,
//     };

//     if exists {
//         // Remove reaction (toggle off)
//         let delete_query = "DELETE FROM voicesphere.topic_reactions WHERE topic_id = ? AND user_id = ? AND emoji = ?";
//         if let Err(e) = data.session.query(delete_query, (topic_id, &req.user_id, &req.emoji)).await {
//             println!("Error removing reaction: {:?}", e);
//             return HttpResponse::InternalServerError().finish();
//         }
//     } else {
//         // Add reaction
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
