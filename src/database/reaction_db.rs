use std::sync::Arc;

use scylla::{batch::Batch, statement::Consistency, Session};
use crate::handlers::reactions::ReactionModel;

const INCREMENT_LIKES_QUERY: &str =
    "UPDATE voicesphere.feeds SET reaction_count = reaction_count + 1 WHERE feed_id = ?";
const DECREMENT_LIKES_QUERY: &str =
    "UPDATE voicesphere.feeds SET reaction_count = reaction_count - 1 WHERE feed_id = ?";

const INCREMENT_LIKES_QUERY_GLOBAL_FEEDS: &str =
    "UPDATE voicesphere.global_feeds SET reaction_count = reaction_count + 1 WHERE feed_id = ?";
const DECREMENT_LIKES_QUERY_GLOBAL_FEEDS: &str =
    "UPDATE voicesphere.global_feeds SET reaction_count = reaction_count - 1 WHERE feed_id = ?";

const INSERT_REACTION_QUERY: &str = "INSERT INTO voicesphere.reactions (feed_id, user_id, reaction_type, reacted_at) VALUES (?, ?, ?, ?) IF NOT EXISTS";
const DELETE_REACTION_QUERY: &str =
    "DELETE FROM voicesphere.reactions WHERE feed_id = ? AND user_id = ?";

// User stats updates
const INCREMENT_LIKES_GIVEN_QUERY: &str =
    "UPDATE voicesphere.user_stats SET likes_given = likes_given + 1 WHERE user_id = ?";
const DECREMENT_LIKES_GIVEN_QUERY: &str =
    "UPDATE voicesphere.user_stats SET likes_given = likes_given - 1 WHERE user_id = ?";
const INCREMENT_LIKES_RECEIVED_QUERY: &str =
    "UPDATE voicesphere.user_stats SET likes_received = likes_received + 1 WHERE user_id = ?";
const DECREMENT_LIKES_RECEIVED_QUERY: &str =
    "UPDATE voicesphere.user_stats SET likes_received = likes_received - 1 WHERE user_id = ?";

pub struct ReactionDB;

impl ReactionDB {
    pub async fn add_reaction(
        session: &Arc<Session>,
        reaction_data: ReactionModel,
    ) -> Result<bool, String> {
        log::info!(
            "Adding reaction: user={} -> feed={} (author={})",
            reaction_data.user_id,
            reaction_data.feed_id,
            reaction_data.author_id
        );

        let mut batch: Batch = Default::default();
        batch.append_statement(INCREMENT_LIKES_QUERY);
        batch.append_statement(INCREMENT_LIKES_QUERY_GLOBAL_FEEDS);
        batch.append_statement(INSERT_REACTION_QUERY);
        batch.append_statement(INCREMENT_LIKES_GIVEN_QUERY);
        batch.append_statement(INCREMENT_LIKES_RECEIVED_QUERY);

        batch.set_consistency(Consistency::One);

        let prepared_batch = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare batch: {}", e))?;
        
        let batch_values = (
            (reaction_data.feed_id.clone(),),
            (reaction_data.feed_id.clone(),),
            (
                reaction_data.feed_id.clone(),
                reaction_data.user_id.clone(),
                reaction_data.reaction_type,
                reaction_data.reacted_at,
            ),
            (reaction_data.user_id.clone(),),      // likes_given for reactor
            (reaction_data.author_id.clone(),),    // likes_received for feed author
        );

        session
            .batch(&prepared_batch, batch_values)
            .await
            .map_err(|e| format!("Error occurred while adding reaction: {}", e))?;

        log::info!(
            "Reaction added successfully: user={} -> feed={}",
            reaction_data.user_id,
            reaction_data.feed_id
        );
        Ok(true)
    }

    pub async fn remove_reaction(
        session: &Arc<Session>,
        reaction_data: ReactionModel,
    ) -> Result<bool, String> {
        log::info!(
            "Removing reaction: user={} -> feed={} (author={})",
            reaction_data.user_id,
            reaction_data.feed_id,
            reaction_data.author_id
        );

        let mut batch: Batch = Default::default();
        batch.append_statement(DECREMENT_LIKES_QUERY);
        batch.append_statement(DECREMENT_LIKES_QUERY_GLOBAL_FEEDS);
        batch.append_statement(DELETE_REACTION_QUERY);
        batch.append_statement(DECREMENT_LIKES_GIVEN_QUERY);
        batch.append_statement(DECREMENT_LIKES_RECEIVED_QUERY);

        batch.set_consistency(Consistency::One);

        let prepared_batch = session
            .prepare_batch(&batch)
            .await
            .map_err(|e| format!("Failed to prepare batch: {}", e))?;
        
        let batch_values = (
            (reaction_data.feed_id.clone(),),
            (reaction_data.feed_id.clone(),),
            (reaction_data.feed_id.clone(), reaction_data.user_id.clone()),
            (reaction_data.user_id.clone(),),      // likes_given for reactor
            (reaction_data.author_id.clone(),),    // likes_received for feed author
        );

        session
            .batch(&prepared_batch, batch_values)
            .await
            .map_err(|e| format!("Error occurred while removing reaction: {}", e))?;

        log::info!(
            "Reaction removed successfully: user={} -> feed={}",
            reaction_data.user_id,
            reaction_data.feed_id
        );
        Ok(true)
    }
}
