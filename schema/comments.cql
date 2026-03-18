CREATE TABLE IF NOT EXISTS voicesphere.comments (
    feed_id    UUID,
    comment_id UUID,  -- UUIDv7: time-ordered + always unique, so ORDER BY comment_id DESC = newest first
    user_id    TEXT,
    comment    TEXT,
    commented_at TIMESTAMP,

    parent_comment_id UUID,  -- NULL for top-level comments
    parent_user_id    TEXT,  -- user who wrote the parent comment

    PRIMARY KEY ((feed_id), comment_id)
) WITH CLUSTERING ORDER BY (comment_id DESC);
