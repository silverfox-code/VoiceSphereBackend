// API Handlers/Routes
pub mod auth;
pub mod chat;
pub mod comments;
pub mod feed;
pub mod profile;
pub mod reactions;
pub mod user;

use axum::Router;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(auth::routes())
        // .merge(user::routes())
        .merge(feed::routes())
        // .merge(chat::routes())
        .merge(profile::routes())
        .merge(reactions::routes())
        .merge(comments::routes())
}
