CREATE KEYSPACE IF NOT EXISTS voicesphere WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };

CREATE TABLE IF NOT EXISTS voicesphere.users (
    id text PRIMARY KEY,

    username text,
    display_name text,
    email text,
    email_verified boolean,

    bio text,
    avatar_url text,
    language text,
    timezone text,

    is_active boolean,
    is_shadow_banned boolean,
    report_count int,

    session_version int,
    last_login_at int,

    created_at int,
    updated_at int
);

CREATE TABLE IF NOT EXISTS voicesphere.topics (
    id uuid PRIMARY KEY,
    title text,
    description text,
    author text,
    created_at timestamp
);

CREATE TABLE IF NOT EXISTS voicesphere.comments (
    topic_id uuid,
    created_at timestamp,
    id uuid,
    user_id text,
    content text,
    PRIMARY KEY (topic_id, created_at, id)
) WITH CLUSTERING ORDER BY (created_at ASC);

CREATE TABLE IF NOT EXISTS voicesphere.reactions (
    comment_id uuid,
    user_id text,
    emoji text,
    PRIMARY KEY (comment_id, user_id)
);
