--- comments on a topic
CREATE TABLE IF NOT EXISTS voicesphere.comments{
    feed_id TIMEUUID,
    comment_id TIMEUUID,
    user_id TEXT,
    comment TEXT,
    commented_at TIMESTAMP,

    parent_comment_id TIMEUUID,  --- NULL for top-level comments, set for replies
    parent_user_id TEXT,          --- user who wrote the parent comment

    PRIMARY KEY ((feed_id), comment_id)
} WITH CLUSTERING ORDER BY (comment_id DESC);
