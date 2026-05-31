use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::api::handlers;
use crate::db::Repository;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Repository>,
}

/// Create the API router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/status", get(handlers::status))
        .route("/feeds", get(handlers::get_feeds).post(handlers::add_feed))
        .route("/feeds/{id}", delete(handlers::delete_feed))
        .route("/items", get(handlers::get_items))
        .route("/items/{id}", get(handlers::get_item))
        .route("/fetch-now", post(handlers::fetch_now))
        .route("/stats", get(handlers::get_stats))
        .with_state(state)
}
