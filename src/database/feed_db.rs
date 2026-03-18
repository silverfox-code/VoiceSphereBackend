use std::sync::Arc;

use scylla::{
    client::session::Session,
    response::PagingState,
    statement::{batch::Batch, Consistency},
    DeserializeRow,
};
use uuid::Uuid;

use crate::models::feed::FeedData;

// ============================================================================
// DB row types — ScyllaDB derives only, never exposed outside this module
// ============================================================================

#[derive(Debug, DeserializeRow)]
struct FeedRow {
    pub feed_id: Uuid,
    pub author_id: String,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_active: bool,
    pub is_restricted: bool,
}

#[derive(Debug, DeserializeRow)]
struct FeedCounts {
    pub comment_count: i64,
    pub reaction_count: i64,
}

fn to_feed_data(row: FeedRow) -> FeedData {
    FeedData {
        id: row.feed_id,
        author: row.author_id,
        author_name: row.author_name,
        author_avatar_url: row.author_avatar_url,
        content: row.content,
        created_at: row.created_at,
        updated_at: row.updated_at,
        like_count: 0,
        comment_count: 0,
        is_liked_by_user: false,
    }
}

// ============================================================================
// Queries
// ============================================================================

const CREATE_FEED_QUERY: &str = "
    INSERT INTO voicesphere.user_feed
        (feed_id, author_id, author_name, author_avatar_url, content, created_at, updated_at, is_active, is_restricted)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

const CREATE_GLOBAL_FEED_QUERY: &str = "
    INSERT INTO voicesphere.global_feed
        (bucket_id, feed_id, author_id, author_name, author_avatar_url, content, created_at, updated_at, is_active, is_restricted)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

// bucket_id = today, feed_id is UUIDv7 so ORDER BY feed_id DESC = newest first
const GET_GLOBAL_FEED_QUERY: &str = "
    SELECT feed_id, author_id, author_name, author_avatar_url, content,
           created_at, updated_at, is_active, is_restricted
    FROM voicesphere.global_feed
    WHERE bucket_id = ?
    ORDER BY feed_id DESC
    LIMIT ?";

const GET_USER_FEED_QUERY: &str = "
    SELECT feed_id, author_id, author_name, author_avatar_url, content,
           created_at, updated_at, is_active, is_restricted
    FROM voicesphere.user_feed
    WHERE author_id = ?
    ORDER BY feed_id DESC
    LIMIT ?";

// bucket_id derived from feed_id (UUIDv7), so single-post lookup needs no extra table
const GET_FEED_DETAILS_QUERY: &str = "
    SELECT feed_id, author_id, author_name, author_avatar_url, content,
           created_at, updated_at, is_active, is_restricted
    FROM voicesphere.global_feed
    WHERE bucket_id = ? AND feed_id = ?";

const GET_FEED_COUNTS_QUERY: &str =
    "SELECT comment_count, reaction_count FROM voicesphere.feed_counts WHERE feed_id = ?";

const CHECK_REACTION_QUERY: &str =
    "SELECT user_id FROM voicesphere.reactions WHERE feed_id = ? AND user_id = ?";

// PK: (author_id, feed_id)
const UPDATE_USER_FEED_QUERY: &str =
    "UPDATE voicesphere.user_feed SET content = ?, updated_at = ? WHERE author_id = ? AND feed_id = ?";

// PK: (bucket_id, feed_id)
const UPDATE_GLOBAL_FEED_QUERY: &str =
    "UPDATE voicesphere.global_feed SET content = ?, updated_at = ? WHERE bucket_id = ? AND feed_id = ?";

// PK: (author_id, feed_id)
const DELETE_USER_FEED_QUERY: &str =
    "DELETE FROM voicesphere.user_feed WHERE author_id = ? AND feed_id = ?";

// PK: (bucket_id, feed_id)
const DELETE_GLOBAL_FEED_QUERY: &str =
    "DELETE FROM voicesphere.global_feed WHERE bucket_id = ? AND feed_id = ?";

// ============================================================================
// Public API
// ============================================================================

pub struct FeedDB;

impl FeedDB {
    pub async fn create_feed(
        session: &Arc<Session>,
        feed: &FeedData,
        author_id: &str,
    ) -> Result<(), String> {
        log::info!("Creating feed: feed_id={}, author={}", feed.id, author_id);

        let bucket_id = bucket_id_from_uuid7(feed.id);

        let mut batch: Batch = Default::default();
        batch.append_statement(CREATE_FEED_QUERY);
        batch.append_statement(CREATE_GLOBAL_FEED_QUERY);
        batch.set_consistency(Consistency::One);

        let prepared = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare create batch: {}", e))?;

        session
            .batch(
                &prepared,
                (
                    (
                        feed.id,
                        author_id,
                        feed.author_name.as_str(),
                        feed.author_avatar_url.as_deref(),
                        feed.content.as_str(),
                        feed.created_at,
                        feed.updated_at,
                        true,
                        false,
                    ),
                    (
                        bucket_id.as_str(),
                        feed.id,
                        author_id,
                        feed.author_name.as_str(),
                        feed.author_avatar_url.as_deref(),
                        feed.content.as_str(),
                        feed.created_at,
                        feed.updated_at,
                        true,
                        false,
                    ),
                ),
            )
            .await
            .map_err(|e| format!("Failed to execute create batch: {}", e))?;

        log::info!("Feed created: feed_id={}", feed.id);
        Ok(())
    }

    /// Fetch the global feed. Queries buckets day-by-day until `limit` posts are
    /// collected or 30 days back have been exhausted.
    pub async fn get_feed(
        session: &Arc<Session>,
        viewer_id: &str,
        limit: i32,
    ) -> Result<Vec<FeedData>, String> {
        let mut feeds: Vec<FeedData> = Vec::new();

        let stmt = session
            .prepare(GET_GLOBAL_FEED_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare global feed query: {}", e))?;

        for days_back in 0..30i64 {
            if feeds.len() >= limit as usize {
                break;
            }

            let bucket = bucket_for_days_back(days_back);
            let remaining = (limit as usize - feeds.len()) as i32;

            let (res, _) = session
                .execute_single_page(&stmt, (bucket.as_str(), remaining), PagingState::start())
                .await
                .map_err(|e| format!("Failed to query bucket {}: {}", bucket, e))?;

            let rows = res
                .into_rows_result()
                .map_err(|e| format!("Failed to read bucket result: {}", e))?;

            let count_before = feeds.len();
            for row in rows
                .rows::<FeedRow>()
                .map_err(|e| format!("Failed to deserialize feed rows: {}", e))?
            {
                let feed = row.map_err(|e| format!("Failed to parse feed row: {}", e))?;
                feeds.push(to_feed_data(feed));
            }

            log::info!(
                "Bucket {} returned {} rows",
                bucket,
                feeds.len() - count_before
            );
        }

        enrich_with_counts(session, &mut feeds, viewer_id).await?;
        Ok(feeds)
    }

    pub async fn get_user_feeds(
        session: &Arc<Session>,
        author_id: &str,
        viewer_id: &str,
        limit: i32,
    ) -> Result<Vec<FeedData>, String> {
        let mut stmt = session
            .prepare(GET_USER_FEED_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare user feed query: {}", e))?;
        stmt.set_page_size(50);

        let (res, _) = session
            .execute_single_page(&stmt, (author_id, limit), PagingState::start())
            .await
            .map_err(|e| format!("Failed to execute user feed query: {}", e))?;

        let rows = res
            .into_rows_result()
            .map_err(|e| format!("Failed to read user feed result: {}", e))?;

        log::info!(
            "User feed for author={} returned {} rows",
            author_id,
            rows.rows_num()
        );

        let mut feeds: Vec<FeedData> = rows
            .rows::<FeedRow>()
            .map_err(|e| format!("Failed to deserialize feed rows: {}", e))?
            .map(|r| {
                r.map(to_feed_data)
                    .map_err(|e| format!("Failed to parse feed row: {}", e))
            })
            .collect::<Result<Vec<_>, _>>()?;

        enrich_with_counts(session, &mut feeds, viewer_id).await?;
        Ok(feeds)
    }

    pub async fn get_feed_by_id(
        session: &Arc<Session>,
        feed_id: Uuid,
        viewer_id: &str,
    ) -> Result<Option<FeedData>, String> {
        let bucket_id = bucket_id_from_uuid7(feed_id);

        let stmt = session
            .prepare(GET_FEED_DETAILS_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare feed details query: {}", e))?;

        let (res, _) = session
            .execute_single_page(&stmt, (bucket_id.as_str(), feed_id), PagingState::start())
            .await
            .map_err(|e| format!("Failed to execute feed details query: {}", e))?;

        let rows = res
            .into_rows_result()
            .map_err(|e| format!("Failed to read feed details result: {}", e))?;

        let mut iter = rows
            .rows::<FeedRow>()
            .map_err(|e| format!("Failed to deserialize feed row: {}", e))?;

        match iter
            .next()
            .transpose()
            .map_err(|e| format!("Failed to parse feed row: {}", e))?
        {
            None => Ok(None),
            Some(row) => {
                let mut feeds = vec![to_feed_data(row)];
                enrich_with_counts(session, &mut feeds, viewer_id).await?;
                Ok(feeds.into_iter().next())
            }
        }
    }

    pub async fn update_feed(
        session: &Arc<Session>,
        feed_id: Uuid,
        author_id: &str,
        content: &str,
        updated_at: i64,
    ) -> Result<(), String> {
        let bucket_id = bucket_id_from_uuid7(feed_id);

        let mut batch: Batch = Default::default();
        batch.append_statement(UPDATE_USER_FEED_QUERY);
        batch.append_statement(UPDATE_GLOBAL_FEED_QUERY);
        batch.set_consistency(Consistency::One);

        let prepared = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare update batch: {}", e))?;

        session
            .batch(
                &prepared,
                (
                    (content, updated_at, author_id, feed_id),
                    (content, updated_at, bucket_id.as_str(), feed_id),
                ),
            )
            .await
            .map_err(|e| format!("Failed to execute update batch: {}", e))?;

        log::info!("Feed updated: feed_id={}", feed_id);
        Ok(())
    }

    pub async fn delete_feed(
        session: &Arc<Session>,
        feed_id: Uuid,
        author_id: &str,
    ) -> Result<(), String> {
        let bucket_id = bucket_id_from_uuid7(feed_id);

        let mut batch: Batch = Default::default();
        batch.append_statement(DELETE_USER_FEED_QUERY);
        batch.append_statement(DELETE_GLOBAL_FEED_QUERY);
        batch.set_consistency(Consistency::One);

        let prepared = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare delete batch: {}", e))?;

        session
            .batch(
                &prepared,
                ((author_id, feed_id), (bucket_id.as_str(), feed_id)),
            )
            .await
            .map_err(|e| format!("Failed to execute delete batch: {}", e))?;

        log::info!("Feed deleted: feed_id={}", feed_id);
        Ok(())
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Enriches a list of FeedData with counts from feed_counts and liked status
/// from reactions. Runs per-feed queries sequentially — acceptable for page sizes ≤50.
async fn enrich_with_counts(
    session: &Arc<Session>,
    feeds: &mut Vec<FeedData>,
    viewer_id: &str,
) -> Result<(), String> {
    if feeds.is_empty() {
        return Ok(());
    }

    let counts_stmt = session
        .prepare(GET_FEED_COUNTS_QUERY)
        .await
        .map_err(|e| format!("Failed to prepare feed counts query: {}", e))?;

    let reaction_stmt = session
        .prepare(CHECK_REACTION_QUERY)
        .await
        .map_err(|e| format!("Failed to prepare reaction check query: {}", e))?;

    for feed in feeds.iter_mut() {
        // Counts
        if let Ok((res, _)) = session
            .execute_single_page(&counts_stmt, (feed.id,), PagingState::start())
            .await
        {
            if let Ok(rows) = res.into_rows_result() {
                if let Ok(mut iter) = rows.rows::<FeedCounts>() {
                    if let Some(Ok(counts)) = iter.next() {
                        feed.comment_count = counts.comment_count as i32;
                        feed.like_count = counts.reaction_count as i32;
                    }
                }
            }
        }

        // Liked by viewer
        if let Ok((res, _)) = session
            .execute_single_page(&reaction_stmt, (feed.id, viewer_id), PagingState::start())
            .await
        {
            if let Ok(rows) = res.into_rows_result() {
                feed.is_liked_by_user = rows.rows_num() > 0;
            }
        }
    }

    Ok(())
}

/// Derives the bucket key ("YYYY-MM-DD") from a UUIDv7.
/// UUIDv7 encodes a Unix timestamp (ms) in the first 48 bits.
fn bucket_id_from_uuid7(id: Uuid) -> String {
    let b = id.as_bytes();
    let ms = ((b[0] as i64) << 40)
        | ((b[1] as i64) << 32)
        | ((b[2] as i64) << 24)
        | ((b[3] as i64) << 16)
        | ((b[4] as i64) << 8)
        | (b[5] as i64);
    chrono::DateTime::from_timestamp_millis(ms)
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%d")
        .to_string()
}

fn bucket_for_days_back(days: i64) -> String {
    (chrono::Utc::now() - chrono::Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}
