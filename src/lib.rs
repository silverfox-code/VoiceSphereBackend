// Library root - module declarations
pub mod models;
pub mod handlers;
pub mod services;
pub mod database;
pub mod websocket;
pub mod middleware;
pub mod config;
pub mod utils;
pub mod state;
pub mod errors;

pub use config::{AppConfig, DatabaseConfig};
// pub use handlers::configure as configure_handlers;
pub use models::*;
pub use services::*;
pub use utils::*;
pub use errors::*;