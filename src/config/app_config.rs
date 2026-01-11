// Application configuration
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub google_client_id: String,
    pub jwt_secret: String,
    pub jwt_expiry: i64, // in seconds  
}

impl AppConfig {
    pub fn from_env() -> Self {
        AppConfig {
            host: std::env::var("APP_HOST").unwrap(),
            port: std::env::var("APP_PORT")
                .unwrap()
                .parse()
                .unwrap(),
            environment: std::env::var("ENVIRONMENT").unwrap(),
            jwt_secret: std::env::var("JWT_SECRET").unwrap(),
            jwt_expiry: std::env::var("JWT_EXPIRY")
                .unwrap()
                .parse()
                .unwrap(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap(),
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
