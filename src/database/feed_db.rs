use std::{ops::ControlFlow, sync::Arc};

use futures_util::TryStreamExt;
use scylla::{
    batch::Batch, prepared_statement, statement::Consistency, transport::PagingState, Session,
};

use crate::handlers::feed::FeedData;

const CREATE_FEED_QUERY: &str = "INSERT INTO voicesphere.feeds (
    feed_id,
    created_at,
    updated_at,
    content,

    author_id,
    author_name,
    author_avatar_url,

    reaction_count,
    comment_count,
    is_liked,
    is_commented,
    reaction_type,
    is_active,
    is_restricted
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const CREATE_GLOBAL_FEED_QUERY: &str = "INSERT INTO voicesphere.global_feed (
    bucket_id,
    feed_id,
    created_at,
    updated_at,
    content,

    author_id,
    author_name,
    author_avatar_url,

    reaction_count,
    comment_count,
    is_liked,
    is_commented,
    reaction_type,
    is_active,
    is_restricted
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

// fetch
// Query to fetch feeds from a single bucket (used in multi-bucket strategy)
// This will be called multiple times with different bucket_ids until we have enough feeds
// Flow: Query "2026-02-12" LIMIT 50 → if not enough → Query "2026-02-11" LIMIT 50 → etc.
// The LIMIT prevents fetching entire buckets (which could be thousands of feeds)
const GET_GLOBAL_FEED_QUERY: &str = "SELECT
    feed_id,
    created_at,
    updated_at,
    content,
    author_id,
    author_name,
    author_avatar_url,
    reaction_count,
    comment_count,
    is_liked,
    is_commented,
    reaction_type,
    is_active,
    is_restricted 
FROM voicesphere.global_feed 
WHERE bucket_id = ? 
ORDER BY created_at DESC
LIMIT ?";

// Query to fetch feeds from a specific user
const GET_USER_FEED_QUERY: &str = "SELECT
    feed_id,
    created_at,
    updated_at,
    content,
    author_id,
    author_name,
    author_avatar_url,
    reaction_count,
    comment_count,
    is_liked,
    is_commented,
    reaction_type,
    is_active,
    is_restricted 
FROM voicesphere.user_feed 
WHERE author_id = ? 
ORDER BY created_at DESC 
LIMIT ?";

const DELETE_FEED_QUERY: &str = "DELETE FROM voicesphere.feeds WHERE feed_id = ?"; 

pub struct FeedDB;

impl FeedDB {
    pub async fn create_feed(
        session: &Arc<Session>,
        feed: &FeedData,
        author_id: &str,
    ) -> Result<bool, String> {
        log::info!("Creating feed in database: {:?}", feed);

        // Generate bucket_id for global_feed (daily buckets)
        let bucket_id = generate_bucket_id();

        let mut batch: Batch = Default::default();
        batch.append_statement(CREATE_FEED_QUERY);
        batch.append_statement(CREATE_GLOBAL_FEED_QUERY);

        // Consistency::One is atleast one replica is updated - fast
        // data may be inconsistent for a short period of time - max 200 ms
        // for better consistency use QUORUM
        batch.set_consistency(Consistency::One);

        // Prepare all statements in the batch at once
        let prepared_batch: Batch = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Error occurred while preparing batch: {}", e))?;

        // Specify bound values to use with each statement
        let batch_values = (
            (
                feed.id.clone(),
                feed.created_at,
                feed.updated_at,
                feed.content.clone(),
                author_id.to_string(),
                feed.author_name.clone(),
                feed.author_avatar_url.clone(),
                feed.like_count,
                feed.comment_count,
                feed.is_liked_by_user,
                false,
                0,
                true,
                false,
            ),
            (
                bucket_id,
                feed.id.clone(),
                feed.created_at,
                feed.updated_at,
                feed.content.clone(),
                author_id.to_string(),
                feed.author_name.clone(),
                feed.author_avatar_url.clone(),
                feed.like_count,
                feed.comment_count,
                feed.is_liked_by_user,
                false,
                0,
                true,
                false,
            ),
        );

        // Run the prepared batch
        session
            .batch(&prepared_batch, batch_values)
            .await
            .map_err(|e| format!("Error occurred while updating batch: {}", e))?;

        log::info!("Feed created successfully in both tables");
        Ok(true)
    }

    pub async fn get_feed(session: &Arc<Session>) -> Result<Vec<FeedData>, String> {
        let mut prepared_statement = session
            .prepare(GET_GLOBAL_FEED_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare get feed query: {}", e))?; // TODO: Implement multi-bucket strategy to fetch feeds until we have enough data
        prepared_statement.set_page_size(50);

        let mut paging_state = PagingState::start();

        let (res, paging_state_response) = session
            .execute_single_page(&prepared_statement, &[], paging_state)
            .await
            .map_err(|e| format!("error occured {}", e))?;

        println!(
            "Paging state response from the prepared statement execution: {:#?} ({:?} rows)",
            paging_state_response,
            res.rows_num()
        );

        let mut feed_list: Vec<FeedData> = Vec::new();

        for row in res
            .rows_typed::<FeedData>()
            .map_err(|e| format!("Failed to deserialize feed data: {}", e))?
        {
            let feed = row.map_err(|e| format!("Failed to parse feed data: {}", e))?;
            log::info!("Fetched feed: {:?}", feed);
            feed_list.push(feed);
        }

        // let rows_res = res
        //     .rows_typed::<FeedData>()
        //     .map_err(|e| format!("Failed to deserialize feed data: {}", e))?;

        // match paging_state_response.into_paging_control_flow() {
        //     ControlFlow::Break(()) => {
        //         log::info!("No more pages to fetch, reached the end of the result set.");
        //     }
        //     ControlFlow::Continue(new_paging_state) => {
        //         // Update paging state from the response, so that query
        //         // will be resumed from where it ended the last time.
        //         paging_state = new_paging_state
        //     }
        // }

        // let feed_list = rows_res.into_iter();

        Ok(feed_list)
    }

    pub async fn get_user_feeds(
        session: &Arc<Session>,
        user_id: &str,
        limit: i32,
    ) -> Result<Vec<FeedData>, String> {
        let mut prepared_statement = session
            .prepare(GET_USER_FEED_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare get user feed query: {}", e))?;

        prepared_statement.set_page_size(50);

        let paging_state = PagingState::start();

        let (res, paging_state_response) = session
            .execute_single_page(&prepared_statement, (user_id, limit), paging_state)
            .await
            .map_err(|e| format!("Error occurred: {}", e))?;

        log::info!(
            "Paging state: {:#?} ({:?} rows)",
            paging_state_response,
            res.rows_num()
        );

        let mut feed_list: Vec<FeedData> = Vec::new();

        for row in res
            .rows_typed::<FeedData>()
            .map_err(|e| format!("Failed to deserialize feed data: {}", e))?
        {
            let feed = row.map_err(|e| format!("Failed to parse feed data: {}", e))?;
            log::info!("Fetched feed: {:?}", feed);
            feed_list.push(feed);
        }

        Ok(feed_list)
    }

    pub async fn delete_feed(session: &Arc<Session>, feed_id: &str) -> Result<bool, String> {
        let prepared_statement = session
            .prepare(DELETE_FEED_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare delete feed query: {}", e))?;

        session
            .execute_unpaged(&prepared_statement, (feed_id,))
            .await
            .map_err(|e| format!("Failed to execute delete feed query: {}", e))?;

        Ok(true)
    }


}

/// Generate bucket_id for global_feed table
/// Returns current date in format: "YYYY-MM-DD"
/// Example: "2026-02-12"
fn generate_bucket_id() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d").to_string()
}
