--- reaction on a topic
CREATE TABLE IF NOT EXISTS voicesphere.reactions{
    feed_id TIMEUUID,
    user_id TEXT, --- user who reacted
    reaction_type INT,
    reacted_at TIMESTAMP,
    PRIMARY KEY ((feed_id), user_id)
};

