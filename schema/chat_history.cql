CREATE TABLE IF NOT EXISTS voicesphere.chat_history{
    user_id TEXT,
    contact_user_id TEXT,

    msg_type TEXT, ---text, audio,image
    msg_content TEXT,
    msg_time TIMESTAMP,

    audio_url TEXT,
    image_url TEXT,

    unread_count INT,
    
    PRIMARY KEY (user_id, contact_user_id)
} WITH CLUSTERING ORDER BY (msg_time DESC);