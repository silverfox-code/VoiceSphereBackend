// Feed handlers
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Router,
};
use scylla::{DeserializeRow, DeserializeValue, FromRow, SerializeRow, SerializeValue};
use serde::{Deserialize, Serialize};
use crate::{AppError, AppResponse, HttpResponse, models::feed, state::AppState};

#[derive(Debug, SerializeRow, DeserializeRow, Clone)]
pub struct FeedResponse{
  pub data: Vec<FeedData>,
}

#[derive(Debug, SerializeRow, DeserializeRow, DeserializeValue, SerializeValue, Clone, FromRow)]
pub struct FeedData{
  pub id: String,
  pub author: String,
  pub author_name: String,
  pub author_avatar_url: Option<String>,
  pub content: String,
  pub created_at: i64,
  pub updated_at: i64,
  pub like_count: i32,
  pub comment_count: i32,
  pub is_liked_by_user: bool,
}

/// Fetch global feed with multi-bucket strategy
/// This queries multiple time buckets until we have enough data
/// Flow: Get ALL feeds from today → if not enough → Get ALL from yesterday → etc.
// pub async fn get_feed(
//     State(state): State<AppState>,
// ) -> Result<HttpResponse<FeedResponse>, AppError> {
//     const REQUESTED_LIMIT: usize = 20; // How many feeds user wants
//     const MAX_BUCKETS_TO_QUERY: usize = 7; // Max 7 days back
    
//     let mut all_feeds: Vec<FeedData> = Vec::new();
    
//     // Generate bucket IDs for the last N days
//     let bucket_ids = generate_recent_bucket_ids(MAX_BUCKETS_TO_QUERY);
    
//     // Query each bucket until we have enough data
//     for bucket_id in bucket_ids {
//         if all_feeds.len() >= REQUESTED_LIMIT {
//             break; // We have enough data, stop querying
//         }
        
//         // Get ALL feeds from this bucket (no LIMIT on individual bucket)
//         let query = "SELECT feed_id, created_at, updated_at, content, author_id, 
//                      author_name, author_avatar_url, reaction_count, comment_count,
//                      is_liked, is_commented, reaction_type, is_active, is_restricted
//                      FROM voicesphere.global_feed 
//                      WHERE bucket_id = ?";
        
//         let result = state.db
//             .query(query, (&bucket_id,))
//             .await
//             .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
//         // If no rows in this bucket, continue to next bucket
//         let rows = match result.rows {
//             Some(rows) => rows,
//             None => continue, // Empty bucket, try next day
//         };
        
//         // Parse ALL rows from this bucket into FeedData
//         for row in rows {
//             // Only include active, non-restricted feeds
//             let is_active: bool = row.columns[12].as_ref()
//                 .and_then(|v| v.as_boolean()).unwrap_or(true);
//             let is_restricted: bool = row.columns[13].as_ref()
//                 .and_then(|v| v.as_boolean()).unwrap_or(false);
            
//             if !is_active || is_restricted {
//                 continue; // Skip inactive or restricted feeds
//             }
            
//             let feed = FeedData {
//                 id: row.columns[0].as_ref()
//                     .and_then(|v| v.as_timeuuid())
//                     .map(|u| u.to_string())
//                     .unwrap_or_default(),
//                 created_at: row.columns[1].as_ref()
//                     .and_then(|v| v.as_timestamp())
//                     .map(|t| t.num_milliseconds())
//                     .unwrap_or(0),
//                 updated_at: row.columns[2].as_ref()
//                     .and_then(|v| v.as_timestamp())
//                     .map(|t| t.num_milliseconds())
//                     .unwrap_or(0),
//                 content: row.columns[3].as_ref()
//                     .and_then(|v| v.as_text())
//                     .unwrap_or("")
//                     .to_string(),
//                 author: row.columns[4].as_ref()
//                     .and_then(|v| v.as_text())
//                     .unwrap_or("")
//                     .to_string(),
//                 author_name: row.columns[5].as_ref()
//                     .and_then(|v| v.as_text())
//                     .unwrap_or("")
//                     .to_string(),
//                 author_avatar_url: row.columns[6].as_ref()
//                     .and_then(|v| v.as_text())
//                     .map(|s| s.to_string()),
//                 like_count: row.columns[7].as_ref()
//                     .and_then(|v| v.as_int())
//                     .unwrap_or(0),
//                 comment_count: row.columns[8].as_ref()
//                     .and_then(|v| v.as_int())
//                     .unwrap_or(0),
//                 is_liked_by_user: row.columns[9].as_ref()
//                     .and_then(|v| v.as_boolean())
//                     .unwrap_or(false),
//             };
            
//             all_feeds.push(feed);
//         }
//     }
    
//     // Sort all feeds by created_at DESC (most recent first)
//     all_feeds.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
//     // Take only the requested limit
//     all_feeds.truncate(REQUESTED_LIMIT);
    
//     Ok(HttpResponse::new(
//         StatusCode::OK,
//         AppResponse {
//             success: true,
//             data: Some(FeedResponse { data: all_feeds }),
//             message: Some("Feeds retrieved successfully".to_string()),
//             error: None,
//         }
//     ))
// }

/// Generate bucket IDs for the last N days
/// Returns: ["2026-02-12", "2026-02-11", "2026-02-10", ...]
fn generate_recent_bucket_ids(days: usize) -> Vec<String> {
    use chrono::{Utc, Duration};
    
    let mut buckets = Vec::with_capacity(days);
    let now = Utc::now();
    
    for i in 0..days {
        let date = now - Duration::days(i as i64);
        buckets.push(date.format("%Y-%m-%d").to_string());
    }
    
    buckets
}

pub async fn get_user_feed() -> Result< HttpResponse<()>, AppError> {
 return Ok(HttpResponse::new(
     StatusCode::NO_CONTENT,
     AppResponse {     
        success: true,
        data: None,
        message: Some("Feed updated successfully".to_string()),
        error: None,
    }));
}

pub async fn create_feed() -> Result< HttpResponse<()>, AppError> {
 return Ok(HttpResponse::new(
     StatusCode::NO_CONTENT,
     AppResponse {     
        success: true,
        data: None,
        message: Some("Feed created successfully".to_string()),
        error: None,
    }));
}

pub async fn update_feed() -> Result< HttpResponse<()>, AppError> {
 return Ok(HttpResponse::new(
     StatusCode::NO_CONTENT,
     AppResponse {     
        success: true,
        data: None,
        message: Some("Feed updated successfully".to_string()),
        error: None,
    }));
}

pub async fn delete_feed() -> Result<HttpResponse<()>,AppError> {
 
  return Ok(HttpResponse::new(
     StatusCode::NO_CONTENT,
     AppResponse {     
        success: true,
        data: None,
        message: Some("Feed deleted successfully".to_string()),
        error: None,
    }));
}

pub async fn get_feed_details() -> Result<HttpResponse<()>,AppError> {
 
  return Ok(HttpResponse::new(
     StatusCode::NO_CONTENT,
     AppResponse {     
        success: true,
        data: None,
        message: Some("Feed details retrieved successfully".to_string()),
        error: None,
    }));
}
                                                                            
pub fn routes () -> Router<AppState> {
    Router::new()
    // .route("/feed", get(get_feed))
    .route("/feed:user_id", get(get_user_feed))
    .route("/feed/create_feed", post(create_feed))
    .route("/feed/update_feed:feed_id", post(update_feed))
    .route("/feed/get_details:feed_id", get(get_feed_details))
    .route("/feed/delete:feed_id", delete(delete_feed))
}