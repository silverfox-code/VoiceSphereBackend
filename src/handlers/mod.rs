// API Handlers/Routes
pub mod auth;
pub mod user;
pub mod feed;
pub mod chat;
pub mod profile;
pub mod reactions;

use axum::Router;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(auth::routes())
        // .merge(user::routes())
        // .merge(feed::routes())
        // .merge(chat::routes())
        .merge(profile::routes())
}
