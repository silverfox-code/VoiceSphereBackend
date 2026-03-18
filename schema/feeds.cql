-- Per-user feed. Partition by author so all posts by a user live together.
-- feed_id is UUIDv7 (time-ordered) so ORDER BY feed_id DESC gives correct time ordering
-- without needing created_at in the primary key.
-- This also means UPDATE/DELETE only need author_id + feed_id (both available in handlers).
CREATE TABLE IF NOT EXISTS voicesphere.user_feed (
    feed_id     UUID,
    author_id   TEXT,
    author_name TEXT,
    author_avatar_url TEXT,

    content     TEXT,
    created_at  TIMESTAMP,
    updated_at  TIMESTAMP,

    is_active      BOOLEAN,
    is_restricted  BOOLEAN,

    PRIMARY KEY (author_id, feed_id)
) WITH CLUSTERING ORDER BY (feed_id DESC);


-- Global feed bucketed by date to keep partition sizes bounded.
-- Bucket strategy: query today's bucket first, fall back to previous days
-- until the requested page is filled.
-- feed_id is UUIDv7 so ORDER BY feed_id DESC = newest first.
CREATE TABLE IF NOT EXISTS voicesphere.global_feed (
    bucket_id TEXT,
    feed_id   UUID,

    author_id   TEXT,
    author_name TEXT,
    author_avatar_url TEXT,

    content    TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,

    is_active     BOOLEAN,
    is_restricted BOOLEAN,

    PRIMARY KEY (bucket_id, feed_id)
) WITH CLUSTERING ORDER BY (feed_id DESC);
