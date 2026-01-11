// use std::{
//     collections::{HashMap, HashSet},
//     sync::{Arc, Mutex},
// };
// use tokio::sync::mpsc;
// use uuid::Uuid;
// use serde::{Deserialize, Serialize};
// use actix_ws::Session;

// #[derive(Debug, Clone, Deserialize)]
// pub struct WsMessage {
//     pub action: String,
//     pub topic_id: Option<String>,
//     pub content: Option<String>,
// }

// #[derive(Debug, Clone, Serialize)]
// pub struct WsEvent {
//     pub event_type: String,
//     pub data: serde_json::Value,
// }

// type SocketSender = mpsc::UnboundedSender<Result<actix_ws::Message, actix_ws::ProtocolError>>;

// pub struct ChatServer {
//     // Map Topic ID -> Set of Session IDs
//     rooms: HashMap<String, HashSet<Uuid>>,
//     // Map Session ID -> Session Sender
//     sessions: HashMap<Uuid, Session>,
// }

// impl ChatServer {
//     pub fn new() -> Self {
//         Self {
//             rooms: HashMap::new(),
//             sessions: HashMap::new(),
//         }
//     }

//     pub fn connect(&mut self, id: Uuid, session: Session) {
//         self.sessions.insert(id, session);
//     }

//     pub fn disconnect(&mut self, id: Uuid) {
//         self.sessions.remove(&id);
//         for subscribers in self.rooms.values_mut() {
//             subscribers.remove(&id);
//         }
//     }

//     pub fn subscribe(&mut self, session_id: Uuid, topic_id: String) {
//         self.rooms
//             .entry(topic_id)
//             .or_insert_with(HashSet::new)
//             .insert(session_id);
//     }

//     pub async fn broadcast(&self, topic_id: &str, event: WsEvent) {
//         if let Some(subscribers) = self.rooms.get(topic_id) {
//             let msg = serde_json::to_string(&event).unwrap();
//             for &session_id in subscribers {
//                 if let Some(session) = self.sessions.get(&session_id) {
//                     let mut session = session.clone();
//                     let msg = msg.clone();
//                     actix_web::rt::spawn(async move {
//                         let _ = session.text(msg).await;
//                     });
//                 }
//             }
//         }
//     }
// }
