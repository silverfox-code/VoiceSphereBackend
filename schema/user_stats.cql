--- to show prrofile data , add rating slater on
CREATE TABLE IF NOT EXISTS voicesphere.user_stats{
    user_id TEXT,

    topic_count INT,
    call_count INT,

    likes_given INT,
    likes_received INT,

    comments_given INT,
    comments_received INT,

    followers_count INT,
    following_count INT,

    PRIMARY KEY (user_id)
};  