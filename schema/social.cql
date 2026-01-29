CREATE TABLE IF NOT EXISTS voicesphere.following{
    user_id TEXT,
    following_id TEXT,
    followed_at TIMESTAMP,
    PRIMARY KEY (user_id, following_id)
} WITH CLUSTERING ORDER BY (followed_at DESC);

CREATE TABLE IF NOT EXISTS voicesphere.followers{
    user_id TEXT,
    follower_id TEXT,
    followed_at TIMESTAMP,
    PRIMARY KEY (user_id, follower_id)
} WITH CLUSTERING ORDER BY (followed_at DESC);