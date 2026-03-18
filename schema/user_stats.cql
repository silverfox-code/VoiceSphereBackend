-- All count columns are COUNTER type.
-- ScyllaDB requires col = col +/- 1 syntax for atomic increments — this only
-- works on COUNTER columns. A table with all non-PK columns as COUNTER is valid.
CREATE TABLE IF NOT EXISTS voicesphere.user_stats (
    user_id TEXT PRIMARY KEY,

    topic_count         COUNTER,
    call_count          COUNTER,

    likes_given         COUNTER,
    likes_received      COUNTER,

    comments_given      COUNTER,
    comments_received   COUNTER,

    followers_count     COUNTER,
    following_count     COUNTER
);
