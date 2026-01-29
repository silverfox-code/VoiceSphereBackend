--- reaction on a topic
CREATE TABLE IF NOT EXISTS voicesphere.reactions{
    topic_id TIMEUUID,
    user_id TEXT, --- user who reacted
    reaction_type INT,
    reacted_at TIMESTAMP,
    PRIMARY KEY (topic_id )
}WITH CLUSTERING ORDER BY (reacted_at DESC);

CREATE TABLE IF NOT EXISTS voicesphere.comments{
    topic_id TIMEUUID,
    comment_id TIMEUUID,
    user_id TEXT,
    comment TEXT,
    commented_at TIMESTAMP,

    parent_comment_id TIMEUUID,
    parent_user_id TEXT,

    PRIMARY KEY (topic_id )
}WITH CLUSTERING ORDER BY (commented_at ASC);
