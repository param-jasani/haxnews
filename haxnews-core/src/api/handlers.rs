use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use crate::api::responses::{HealthResponse, ItemsListResponse, ItemResponse, ErrorResponse};
use crate::api::routes::AppState;
use crate::models::FeedSource;

#[derive(Deserialize)]
pub struct ItemsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct FetchNowRequest {
    pub feed_id: Option<String>,
}

#[derive(Serialize)]
pub struct FeedsListResponse {
    pub feeds: Vec<FeedSource>,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub feeds_count: usize,
    pub items_count: usize,
}

pub async fn status() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn get_feeds(State(state): State<AppState>) -> Result<Json<FeedsListResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_all_feeds() {
        Ok(feeds) => Ok(Json(FeedsListResponse { feeds })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Database error".to_string(),
            message: Some(e.to_string()),
        })))
    }
}

pub async fn add_feed(
    State(state): State<AppState>,
    Json(feed): Json<FeedSource>,
) -> Result<Json<FeedSource>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.save_feed(&feed) {
        Ok(_) => Ok(Json(feed)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Failed to save feed".to_string(),
            message: Some(e.to_string()),
        })))
    }
}

pub async fn delete_feed(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.db.delete_feed(&id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "Not found".to_string(),
            message: None,
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Failed to delete feed".to_string(),
            message: Some(e.to_string()),
        })))
    }
}

pub async fn get_items(
    State(state): State<AppState>,
    Query(query): Query<ItemsQuery>,
) -> Result<Json<ItemsListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit;
    let offset = query.offset;
    let search = query.search.as_deref();
    let _category = query.category.as_deref();

    match state.db.get_items(limit, offset, search) {
        Ok(items) => {
            let total = items.len();
            let response_items: Vec<ItemResponse> = items.into_iter().map(ItemResponse::from).collect();
            Ok(Json(ItemsListResponse {
                items: response_items,
                total,
            }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Database error".to_string(),
            message: Some(e.to_string()),
        })))
    }
}

pub async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ItemResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_item(&id) {
        Ok(Some(item)) => Ok(Json(ItemResponse::from(item))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "Not found".to_string(),
            message: None,
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Database error".to_string(),
            message: Some(e.to_string()),
        })))
    }
}

pub async fn fetch_now(
    State(_state): State<AppState>,
    Json(_req): Json<FetchNowRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Ideally this would trigger an async task or channel message
    // For now, we'll return ACCEPTED
    Ok(StatusCode::ACCEPTED)
}

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let feeds = state.db.get_all_feeds().unwrap_or_default().len();
    let items = state.db.get_all_items().unwrap_or_default().len();
    Ok(Json(StatsResponse {
        feeds_count: feeds,
        items_count: items,
    }))
}
