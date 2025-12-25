-- Comment reactions table
CREATE TABLE IF NOT EXISTS voicesphere.comment_reactions (
    topic_id UUID,
    comment_id UUID,
    user_id TEXT,
    emoji TEXT,
    created_at TIMESTAMP,
    PRIMARY KEY ((topic_id, comment_id), user_id)
);
