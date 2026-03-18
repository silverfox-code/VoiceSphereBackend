use std::sync::Arc;

use scylla::{
    client::session::Session,
    statement::{
        batch::{Batch, BatchType},
        Consistency,
    },
};

use crate::models::comment::Comment;

const INSERT_COMMENT_QUERY: &str = "
    INSERT INTO voicesphere.comments
        (feed_id, comment_id, user_id, comment, commented_at, parent_comment_id, parent_user_id)
    VALUES (?, ?, ?, ?, ?, ?, ?)";

// comment_id is the clustering key — feed_id + comment_id is the full primary key
const DELETE_COMMENT_QUERY: &str =
    "DELETE FROM voicesphere.comments WHERE feed_id = ? AND comment_id = ?";

// COUNTER updates (feed_counts + user_stats) — must be in a counter batch, separate from regular writes
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

pub struct CommentsDB;

impl CommentsDB {
    pub async fn add_comment(session: &Arc<Session>, comment: Comment) -> Result<(), String> {
        log::info!(
            "Adding comment: user={} -> feed={}, comment_id={}",
            comment.user_id, comment.feed_id, comment.comment_id
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
            .batch(&prepared_write, ((
                comment.feed_id,
                comment.comment_id,
                comment.user_id.clone(),
                comment.comment.clone(),
                comment.commented_at,
                comment.parent_comment_id,
                comment.parent_user_id.clone(),
            ),))
            .await
            .map_err(|e| format!("Failed to insert comment: {}", e))?;

        // Batch 2 (counter): feed_counts + user_stats
        // All three are COUNTER updates so they can share a counter batch
        let mut counter_batch = Batch::new(BatchType::Counter);
        counter_batch.append_statement(INCREMENT_COMMENT_COUNT_QUERY);
        counter_batch.append_statement(INCREMENT_COMMENTS_GIVEN_QUERY);
        counter_batch.append_statement(INCREMENT_COMMENTS_RECEIVED_QUERY);

        let prepared_counter = session
            .prepare_batch(&counter_batch)
            .await
            .map_err(|e| format!("Failed to prepare counter batch: {}", e))?;

        session
            .batch(&prepared_counter, (
                (comment.feed_id,),
                (comment.user_id.as_str(),),
                (comment.author_id.as_str(),),
            ))
            .await
            .map_err(|e| format!("Failed to update counters: {}", e))?;

        log::info!(
            "Comment added: user={} -> feed={}, comment_id={}",
            comment.user_id, comment.feed_id, comment.comment_id
        );
        Ok(())
    }

    pub async fn remove_comment(session: &Arc<Session>, comment: Comment) -> Result<(), String> {
        log::info!(
            "Removing comment: user={} -> feed={}, comment_id={}",
            comment.user_id, comment.feed_id, comment.comment_id
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
            .batch(&prepared_counter, (
                (comment.feed_id,),
                (comment.user_id.as_str(),),
                (comment.author_id.as_str(),),
            ))
            .await
            .map_err(|e| format!("Failed to update counters: {}", e))?;

        log::info!(
            "Comment removed: user={} -> feed={}, comment_id={}",
            comment.user_id, comment.feed_id, comment.comment_id
        );
        Ok(())
    }
}
