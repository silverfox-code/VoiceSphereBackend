
use scylla::Session;
use std::sync::Arc; 

//Shared resource state for the application
#[derive(Clone)]
pub struct AppState{
    pub db : Arc<Session>,
    pub google_client_id: String,
    pub jwt_secret: String,
}

impl AppState{
    pub fn new (db: Arc<Session>, google_client_id: String, jwt_secret: String) -> Self {
        Self { db, google_client_id, jwt_secret }
    }
}