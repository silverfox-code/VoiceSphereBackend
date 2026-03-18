-- Denormalized counters for a feed post.
-- Keyed only by feed_id — no partition key lookup needed to increment/decrement.
-- ScyllaDB only supports col = col +/- 1 on COUNTER columns, so counts cannot
-- live in user_feed / global_feed (which mix regular and counter columns).
CREATE TABLE IF NOT EXISTS voicesphere.feed_counts (
    feed_id        UUID PRIMARY KEY,
    comment_count  COUNTER,
    reaction_count COUNTER
);
