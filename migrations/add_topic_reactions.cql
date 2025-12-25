-- Topic Reactions Table
CREATE TABLE IF NOT EXISTS voicesphere.topic_reactions (
    topic_id uuid,
    user_id text,
    emoji text,
    created_at timestamp,
    PRIMARY KEY (topic_id, user_id, emoji)
);
