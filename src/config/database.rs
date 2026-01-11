// Database configuration
use serde::{Deserialize, Serialize};
use scylla::{Session, SessionBuilder, SessionConfig, transport::session_builder};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub hosts: Vec<String>,
    pub keyspace: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub port: u16,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        DatabaseConfig {
            hosts: std::env::var("DB_HOSTS")
                .unwrap_or_else(|_| "127.0.0.1".to_string())
                .split(',')
                .map(|h| h.to_string())
                .collect(),
            keyspace: std::env::var("DB_KEYSPACE").unwrap_or_else(|_| "voicesphere".to_string()),
            username: std::env::var("DB_USERNAME").ok(),
            password: std::env::var("DB_PASSWORD").ok(),
            port: std::env::var("DB_PORT")
                .unwrap_or_else(|_| "9042".to_string())
                .parse()
                .unwrap_or(9042),
        }
    }

    pub async fn create_session(&self) -> Result<Arc<Session>, Box<dyn std::error::Error>> {
        log::info!("Connecting to Scylla at hosts: {:?}, port: {}", self.hosts, self.port);

        // Build SessionConfig properly
        let mut session_builder = SessionBuilder::new();
        
        // Add known nodes using string addresses
        for host in &self.hosts {
            let node_addr = format!("{}:{}", host, self.port);
            // config.add_known_node(&node_addr);
            session_builder = session_builder.known_node(&node_addr);
        }

        let session = session_builder.build().await?;
        
        // Set keyspace
        session.use_keyspace(&self.keyspace, false).await?;

        log::info!("Connected to Scylla keyspace: {}", self.keyspace);

        Ok(Arc::new(session))
    }
}
