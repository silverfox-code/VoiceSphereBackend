use std::sync::Arc;

use scylla::{
    client::session::Session,
    statement::{
        batch::{Batch, BatchType},
        Consistency,
    },
};

use crate::models::reaction::ReactionModel;

const INSERT_REACTION_QUERY: &str =
    "INSERT INTO voicesphere.reactions (feed_id, user_id, reaction_type, reacted_at) VALUES (?, ?, ?, ?) IF NOT EXISTS";
const DELETE_REACTION_QUERY: &str =
    "DELETE FROM voicesphere.reactions WHERE feed_id = ? AND user_id = ?";

// COUNTER updates (feed_counts + user_stats) — must be in a counter batch, separate from regular writes
const INCREMENT_REACTION_COUNT_QUERY: &str =
    "UPDATE voicesphere.feed_counts SET reaction_count = reaction_count + 1 WHERE feed_id = ?";
const DECREMENT_REACTION_COUNT_QUERY: &str =
    "UPDATE voicesphere.feed_counts SET reaction_count = reaction_count - 1 WHERE feed_id = ?";

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
        reaction: ReactionModel,
    ) -> Result<(), String> {
        log::info!(
            "Adding reaction: user={} -> feed={} (author={})",
            reaction.user_id, reaction.feed_id, reaction.author_id
        );

        // Batch 1 (logged): insert the reaction row (IF NOT EXISTS for idempotency)
        let mut write_batch: Batch = Default::default();
        write_batch.append_statement(INSERT_REACTION_QUERY);
        write_batch.set_consistency(Consistency::One);

        let prepared_write = session
            .prepare_batch(&write_batch)
            .await
            .map_err(|e| format!("Failed to prepare insert batch: {}", e))?;

        session
            .batch(&prepared_write, ((
                reaction.feed_id,
                reaction.user_id.clone(),
                reaction.reaction_type,
                reaction.reacted_at,
            ),))
            .await
            .map_err(|e| format!("Failed to insert reaction: {}", e))?;

        // Batch 2 (counter): feed_counts + user_stats
        let mut counter_batch = Batch::new(BatchType::Counter);
        counter_batch.append_statement(INCREMENT_REACTION_COUNT_QUERY);
        counter_batch.append_statement(INCREMENT_LIKES_GIVEN_QUERY);
        counter_batch.append_statement(INCREMENT_LIKES_RECEIVED_QUERY);

        let prepared_counter = session
            .prepare_batch(&counter_batch)
            .await
            .map_err(|e| format!("Failed to prepare counter batch: {}", e))?;

        session
            .batch(&prepared_counter, (
                (reaction.feed_id,),
                (reaction.user_id.as_str(),),
                (reaction.author_id.as_str(),),
            ))
            .await
            .map_err(|e| format!("Failed to update counters: {}", e))?;

        log::info!(
            "Reaction added: user={} -> feed={}",
            reaction.user_id, reaction.feed_id
        );
        Ok(())
    }

    pub async fn remove_reaction(
        session: &Arc<Session>,
        reaction: ReactionModel,
    ) -> Result<(), String> {
        log::info!(
            "Removing reaction: user={} -> feed={} (author={})",
            reaction.user_id, reaction.feed_id, reaction.author_id
        );

        // Batch 1 (logged): delete the reaction row
        let mut write_batch: Batch = Default::default();
        write_batch.append_statement(DELETE_REACTION_QUERY);
        write_batch.set_consistency(Consistency::One);

        let prepared_write = session
            .prepare_batch(&write_batch)
            .await
            .map_err(|e| format!("Failed to prepare delete batch: {}", e))?;

        session
            .batch(&prepared_write, ((reaction.feed_id, reaction.user_id.clone()),))
            .await
            .map_err(|e| format!("Failed to delete reaction: {}", e))?;

        // Batch 2 (counter): feed_counts + user_stats
        let mut counter_batch = Batch::new(BatchType::Counter);
        counter_batch.append_statement(DECREMENT_REACTION_COUNT_QUERY);
        counter_batch.append_statement(DECREMENT_LIKES_GIVEN_QUERY);
        counter_batch.append_statement(DECREMENT_LIKES_RECEIVED_QUERY);

        let prepared_counter = session
            .prepare_batch(&counter_batch)
            .await
            .map_err(|e| format!("Failed to prepare counter batch: {}", e))?;

        session
            .batch(&prepared_counter, (
                (reaction.feed_id,),
                (reaction.user_id.as_str(),),
                (reaction.author_id.as_str(),),
            ))
            .await
            .map_err(|e| format!("Failed to update counters: {}", e))?;

        log::info!(
            "Reaction removed: user={} -> feed={}",
            reaction.user_id, reaction.feed_id
        );
        Ok(())
    }
}
