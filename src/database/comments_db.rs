use std::sync::Arc;

use scylla::{batch::Batch, statement::Consistency, Session};

use crate::models::comment::Comment;

const INCREMENT_COMMENT_COUNT_QUERY: &str =
    "UPDATE voicesphere.feeds SET comment_count = comment_count + 1 WHERE feed_id = ?";
const DECREMENT_COMMENT_COUNT_QUERY: &str =
    "UPDATE voicesphere.feeds SET comment_count = comment_count - 1 WHERE feed_id = ?";

const INCREMENT_COMMENT_COUNT_GLOBAL_FEEDS: &str =
    "UPDATE voicesphere.global_feeds SET comment_count = comment_count + 1 WHERE feed_id = ?";
const DECREMENT_COMMENT_COUNT_GLOBAL_FEEDS: &str =
    "UPDATE voicesphere.global_feeds SET comment_count = comment_count - 1 WHERE feed_id = ?";

const INSERT_COMMENT_QUERY: &str = "INSERT INTO voicesphere.comments (feed_id, comment_id, user_id, comment, commented_at, parent_comment_id, parent_user_id) VALUES (?, ?, ?, ?, ?, ?, ?)";
const DELETE_COMMENT_QUERY: &str =
    "DELETE FROM voicesphere.comments WHERE feed_id = ? AND comment_id = ?";

// User stats updates
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
    /// Add a comment to a feed and increment comment counters
    pub async fn add_comment(
        session: &Arc<Session>,
        comment_data: Comment,
    ) -> Result<bool, String> {
        log::info!(
            "Adding comment: user={} -> feed={} (author={})",
            comment_data.user_id,
            comment_data.feed_id,
            comment_data.author_id
        );

        let mut batch: Batch = Default::default();
        batch.append_statement(INCREMENT_COMMENT_COUNT_QUERY);
        batch.append_statement(INCREMENT_COMMENT_COUNT_GLOBAL_FEEDS);
        batch.append_statement(INSERT_COMMENT_QUERY);
        batch.append_statement(INCREMENT_COMMENTS_GIVEN_QUERY);
        batch.append_statement(INCREMENT_COMMENTS_RECEIVED_QUERY);

        batch.set_consistency(Consistency::One);

        let prepared_batch = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare batch: {}", e))?;

        let batch_values = (
            (comment_data.feed_id.clone(),),
            (comment_data.feed_id.clone(),),
            (
                comment_data.feed_id.clone(),
                comment_data.comment_id.clone(),
                comment_data.user_id.clone(),
                comment_data.comment.clone(),
                comment_data.commented_at,
                comment_data.parent_comment_id.clone(),
                comment_data.parent_user_id.clone(),
            ),
            (comment_data.user_id.clone(),),       // comments_given for commenter
            (comment_data.author_id.clone(),),     // comments_received for feed author
        );

        session
            .batch(&prepared_batch, batch_values)
            .await
            .map_err(|e| format!("Error occurred while adding comment: {}", e))?;

        log::info!(
            "Comment added successfully: user={} -> feed={}, comment_id={}",
            comment_data.user_id,
            comment_data.feed_id,
            comment_data.comment_id
        );
        Ok(true)
    }

    /// Remove a comment from a feed and decrement comment counters
    pub async fn remove_comment(
        session: &Arc<Session>,
        comment_data: Comment,
    ) -> Result<bool, String> {
        log::info!(
            "Removing comment: user={} -> feed={} (author={}), comment_id={}",
            comment_data.user_id,
            comment_data.feed_id,
            comment_data.author_id,
            comment_data.comment_id
        );

        let mut batch: Batch = Default::default();
        batch.append_statement(DECREMENT_COMMENT_COUNT_QUERY);
        batch.append_statement(DECREMENT_COMMENT_COUNT_GLOBAL_FEEDS);
        batch.append_statement(DELETE_COMMENT_QUERY);
        batch.append_statement(DECREMENT_COMMENTS_GIVEN_QUERY);
        batch.append_statement(DECREMENT_COMMENTS_RECEIVED_QUERY);

        batch.set_consistency(Consistency::One);

        let prepared_batch = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare batch: {}", e))?;

        let batch_values = (
            (comment_data.feed_id.clone(),),
            (comment_data.feed_id.clone(),),
            (comment_data.feed_id.clone(), comment_data.comment_id.clone()),
            (comment_data.user_id.clone(),),       // comments_given for commenter
            (comment_data.author_id.clone(),),     // comments_received for feed author
        );

        session
            .batch(&prepared_batch, batch_values)
            .await
            .map_err(|e| format!("Error occurred while removing comment: {}", e))?;

        log::info!(
            "Comment removed successfully: user={} -> feed={}, comment_id={}",
            comment_data.user_id,
            comment_data.feed_id,
            comment_data.comment_id
        );
        Ok(true)
    }
}