CREATE TABLE IF NOT EXISTS voicesphere.user_feed{
    user_id TEXT,
    topic_id TIMEUUID,
    created_at TIMESTAMP,
    description TEXT,

    author_id TEXT,
    author_name TEXT,
    author_avatar_url TEXT,

    reaction_count INT,
    comment_count INT,

    is_liked BOOLEAN,
    is_commented BOOLEAN,
    reaction_type INT,

    is_active BOOLEAN,
    is_restricted BOOLEAN,
    PRIMARY KEY (user_id, created_at, topic_id)
} WITH CLUSTERING ORDER BY (created_at DESC);

CREATE TABLE IF NOT EXISTS voicesphere.global_feed{
    bucket_id TEXT,
    user_id TEXT,
    topic_id TIMEUUID,
    created_at TIMESTAMP,
    description TEXT,

    author_id TEXT,
    author_name TEXT,
    author_avatar_url TEXT,

    reaction_count INT,
    comment_count INT,

    is_liked BOOLEAN,
    is_commented BOOLEAN,
    reaction_type INT,

    is_active BOOLEAN,
    is_restricted BOOLEAN,
    PRIMARY KEY (bucket_id, created_at, topic_id)
} WITH CLUSTERING ORDER BY (created_at DESC);