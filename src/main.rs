// Main entry point
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use voicesphere_backend::state::AppState;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use voicesphere_backend::config::{AppConfig, DatabaseConfig};
use voicesphere_backend::handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration from environment variables
    let app_config = AppConfig::from_env();
    let db_config = DatabaseConfig::from_env();

    log::info!(
        "Starting VoiceSphere Backend on {}:{}",
        app_config.host,
        app_config.port
    );

    // Initialize Scylla session
    let session = db_config.create_session().await?;
    log::info!("Scylla session initialized successfully");

    // Create shared application state
    let state = AppState { db: session, google_client_id: app_config.google_client_id, jwt_secret: app_config.jwt_secret };

    // Create health check endpoint
    let app = Router::new()
        .nest("/api", handlers::routes())
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        // TODO: Add authentication middleware
        // .layer(middleware::from_fn(AuthLayer))
        .fallback(handle_404)
        .with_state(state);

    let addr = format!("{}:{}", app_config.host, app_config.port);
    let listener = TcpListener::bind(&addr).await?;

    log::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
        
// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// 404 handler
pub async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
