use std::sync::Arc;

use scylla::{
    client::session::Session,
    response::PagingState,
    statement::{
        batch::{Batch, BatchType},
        Consistency,
    },
    DeserializeRow,
};
use uuid::Uuid;

use crate::models::comment::Comment;

// ============================================================================
// DB row type — matches the comments table exactly
// ============================================================================

#[derive(Debug, DeserializeRow)]
struct CommentRow {
    pub feed_id: Uuid,
    pub comment_id: Uuid,
    pub user_id: String,
    pub comment: String,
    pub commented_at: i64,
    pub parent_comment_id: Option<Uuid>,
    pub parent_user_id: Option<String>,
}

fn to_comment(row: CommentRow) -> Comment {
    Comment {
        feed_id: row.feed_id,
        comment_id: row.comment_id,
        user_id: row.user_id,
        author_id: String::new(), // not stored in comments table
        comment: row.comment,
        commented_at: row.commented_at,
        parent_comment_id: row.parent_comment_id,
        parent_user_id: row.parent_user_id,
    }
}

// ============================================================================
// Queries
// ============================================================================

const INSERT_COMMENT_QUERY: &str = "
    INSERT INTO voicesphere.comments
        (feed_id, comment_id, user_id, comment, commented_at, parent_comment_id, parent_user_id)
    VALUES (?, ?, ?, ?, ?, ?, ?)";

// comment_id is the clustering key — feed_id + comment_id is the full primary key
const DELETE_COMMENT_QUERY: &str =
    "DELETE FROM voicesphere.comments WHERE feed_id = ? AND comment_id = ?";

const GET_FEED_COMMENTS_QUERY: &str = "
    SELECT feed_id, comment_id, user_id, comment, commented_at, parent_comment_id, parent_user_id
    FROM voicesphere.comments
    WHERE feed_id = ?
    LIMIT ?";

const GET_COMMENT_QUERY: &str = "
    SELECT feed_id, comment_id, user_id, comment, commented_at, parent_comment_id, parent_user_id
    FROM voicesphere.comments
    WHERE feed_id = ? AND comment_id = ?";

const UPDATE_COMMENT_QUERY: &str =
    "UPDATE voicesphere.comments SET comment = ?, commented_at = ? WHERE feed_id = ? AND comment_id = ?";

// COUNTER updates — must be in a separate counter batch
const INCREMENT_COMMENT_COUNT_QUERY: &str =
    "UPDATE voicesphere.feed_counts SET comment_count = comment_count + 1 WHERE feed_id = ?";
const DECREMENT_COMMENT_COUNT_QUERY: &str =
    "UPDATE voicesphere.feed_counts SET comment_count = comment_count - 1 WHERE feed_id = ?";

const INCREMENT_COMMENTS_GIVEN_QUERY: &str =
    "UPDATE voicesphere.user_stats SET comments_given = comments_given + 1 WHERE user_id = ?";
const DECREMENT_COMMENTS_GIVEN_QUERY: &str =
    "UPDATE voicesphere.user_stats SET comments_given = comments_given - 1 WHERE user_id = ?";
const INCREMENT_COMMENTS_RECEIVED_QUERY: &str =
    "UPDATE voicesphere.user_stats SET comments_received = comments_received + 1 WHERE user_id = ?";
const DECREMENT_COMMENTS_RECEIVED_QUERY: &str =
    "UPDATE voicesphere.user_stats SET comments_received = comments_received - 1 WHERE user_id = ?";

// ============================================================================
// Public API
// ============================================================================

pub struct CommentsDB;

impl CommentsDB {
    pub async fn add_comment(session: &Arc<Session>, comment: Comment) -> Result<(), String> {
        log::info!(
            "Adding comment: user={} -> feed={}, comment_id={}",
            comment.user_id,
            comment.feed_id,
            comment.comment_id
        );

        // Batch 1 (logged): insert the comment row
        let mut write_batch: Batch = Default::default();
        write_batch.append_statement(INSERT_COMMENT_QUERY);
        write_batch.set_consistency(Consistency::One);

        let prepared_write = session
            .prepare_batch(&write_batch)
            .await
            .map_err(|e| format!("Failed to prepare insert batch: {}", e))?;

        session
            .batch(
                &prepared_write,
                ((
                    comment.feed_id,
                    comment.comment_id,
                    comment.user_id.clone(),
                    comment.comment.clone(),
                    comment.commented_at,
                    comment.parent_comment_id,
                    comment.parent_user_id.clone(),
                ),),
            )
            .await
            .map_err(|e| format!("Failed to insert comment: {}", e))?;

        // Batch 2 (counter): feed_counts + user_stats
        let mut counter_batch = Batch::new(BatchType::Counter);
        counter_batch.append_statement(INCREMENT_COMMENT_COUNT_QUERY);
        counter_batch.append_statement(INCREMENT_COMMENTS_GIVEN_QUERY);
        counter_batch.append_statement(INCREMENT_COMMENTS_RECEIVED_QUERY);

        let prepared_counter = session
            .prepare_batch(&counter_batch)
            .await
            .map_err(|e| format!("Failed to prepare counter batch: {}", e))?;

        session
            .batch(
                &prepared_counter,
                (
                    (comment.feed_id,),
                    (comment.user_id.as_str(),),
                    (comment.author_id.as_str(),),
                ),
            )
            .await
            .map_err(|e| format!("Failed to update counters: {}", e))?;

        log::info!(
            "Comment added: user={} -> feed={}, comment_id={}",
            comment.user_id,
            comment.feed_id,
            comment.comment_id
        );
        Ok(())
    }

    pub async fn remove_comment(session: &Arc<Session>, comment: Comment) -> Result<(), String> {
        log::info!(
            "Removing comment: user={} -> feed={}, comment_id={}",
            comment.user_id,
            comment.feed_id,
            comment.comment_id
        );

        // Batch 1 (logged): delete the comment row
        let mut write_batch: Batch = Default::default();
        write_batch.append_statement(DELETE_COMMENT_QUERY);
        write_batch.set_consistency(Consistency::One);

        let prepared_write = session
            .prepare_batch(&write_batch)
            .await
            .map_err(|e| format!("Failed to prepare delete batch: {}", e))?;

        session
            .batch(&prepared_write, ((comment.feed_id, comment.comment_id),))
            .await
            .map_err(|e| format!("Failed to delete comment: {}", e))?;

        // Batch 2 (counter): feed_counts + user_stats
        let mut counter_batch = Batch::new(BatchType::Counter);
        counter_batch.append_statement(DECREMENT_COMMENT_COUNT_QUERY);
        counter_batch.append_statement(DECREMENT_COMMENTS_GIVEN_QUERY);
        counter_batch.append_statement(DECREMENT_COMMENTS_RECEIVED_QUERY);

        let prepared_counter = session
            .prepare_batch(&counter_batch)
            .await
            .map_err(|e| format!("Failed to prepare counter batch: {}", e))?;

        session
            .batch(
                &prepared_counter,
                (
                    (comment.feed_id,),
                    (comment.user_id.as_str(),),
                    (comment.author_id.as_str(),),
                ),
            )
            .await
            .map_err(|e| format!("Failed to update counters: {}", e))?;

        log::info!(
            "Comment removed: user={} -> feed={}, comment_id={}",
            comment.user_id,
            comment.feed_id,
            comment.comment_id
        );
        Ok(())
    }

    pub async fn get_comments(
        session: &Arc<Session>,
        feed_id: Uuid,
        limit: i32,
    ) -> Result<Vec<Comment>, String> {
        let stmt = session
            .prepare(GET_FEED_COMMENTS_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare get comments query: {}", e))?;

        let (res, _) = session
            .execute_single_page(&stmt, (feed_id, limit), PagingState::start())
            .await
            .map_err(|e| format!("Failed to execute get comments query: {}", e))?;

        let rows = res
            .into_rows_result()
            .map_err(|e| format!("Failed to read comments result: {}", e))?;

        log::info!("Fetched {} comments for feed={}", rows.rows_num(), feed_id);

        rows.rows::<CommentRow>()
            .map_err(|e| format!("Failed to deserialize comment rows: {}", e))?
            .map(|r| {
                r.map(to_comment)
                    .map_err(|e| format!("Failed to parse comment row: {}", e))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_comment(
        session: &Arc<Session>,
        feed_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Option<Comment>, String> {
        let stmt = session
            .prepare(GET_COMMENT_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare get comment query: {}", e))?;

        let (res, _) = session
            .execute_single_page(&stmt, (feed_id, comment_id), PagingState::start())
            .await
            .map_err(|e| format!("Failed to execute get comment query: {}", e))?;

        let rows = res
            .into_rows_result()
            .map_err(|e| format!("Failed to read comment result: {}", e))?;

        rows.rows::<CommentRow>()
            .map_err(|e| format!("Failed to deserialize comment row: {}", e))?
            .next()
            .transpose()
            .map(|opt| opt.map(to_comment))
            .map_err(|e| format!("Failed to parse comment row: {}", e))
    }

    pub async fn update_comment(
        session: &Arc<Session>,
        feed_id: Uuid,
        comment_id: Uuid,
        comment_text: &str,
        commented_at: i64,
    ) -> Result<(), String> {
        let stmt = session
            .prepare(UPDATE_COMMENT_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare update comment query: {}", e))?;

        session
            .execute_unpaged(&stmt, (comment_text, commented_at, feed_id, comment_id))
            .await
            .map_err(|e| format!("Failed to update comment: {}", e))?;

        log::info!(
            "Comment updated: feed={}, comment_id={}",
            feed_id,
            comment_id
        );
        Ok(())
    }
}
