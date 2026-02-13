--- should i keep single table for all feeds?

CREATE TABLE IF NOT EXISTS voicesphere.user_feed{
    feed_id TIMEUUID,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    content TEXT,

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
    PRIMARY KEY (author_id, created_at, feed_id)
} WITH CLUSTERING ORDER BY (created_at DESC);

CREATE TABLE IF NOT EXISTS voicesphere.global_feed{
    bucket_id TEXT,

    feed_id TIMEUUID,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    content TEXT,

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
    PRIMARY KEY (bucket_id, created_at, feed_id)
} WITH CLUSTERING ORDER BY (created_at DESC);