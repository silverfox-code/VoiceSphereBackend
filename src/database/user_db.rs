use chrono;
use scylla::{client::session::Session, DeserializeRow};
use std::sync::Arc;

use crate::{models::user::UserStatsData, User};

// ============================================================================
// DB row type for user_stats COUNTER table
// ============================================================================

#[derive(Debug, DeserializeRow)]
struct UserStatsRow {
    pub user_id: String,
    pub topic_count: i64,
    pub call_count: i64,
    pub likes_given: i64,
    pub likes_received: i64,
    pub comments_given: i64,
    pub comments_received: i64,
    pub followers_count: i64,
    pub following_count: i64,
}

// ============================================================================
// Queries
// ============================================================================

const CREATE_USER_QUERY: &str = "INSERT INTO voicesphere.users (
    id,
    device_id,
    username,
    display_name,
    email,
    email_verified,
    bio,
    avatar_url,
    language,
    timezone,
    is_active,
    is_shadow_banned,
    report_count,
    session_version,
    last_login_at,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const GET_USER_QUERY: &str = "SELECT
    id,
    device_id,
    username,
    display_name,
    email,
    email_verified,
    bio,
    avatar_url,
    language,
    timezone,
    is_active,
    is_shadow_banned,
    report_count,
    session_version,
    last_login_at,
    created_at,
    updated_at
FROM voicesphere.users
WHERE id = ?";

const GET_USER_STATS_QUERY: &str = "SELECT
    user_id,
    topic_count, call_count,
    likes_given, likes_received,
    comments_given, comments_received,
    followers_count, following_count
FROM voicesphere.user_stats
WHERE user_id = ?";

const UPDATE_LAST_LOGIN_QUERY: &str = "UPDATE voicesphere.users SET last_login_at = ? WHERE id = ?";
const UPDATE_USERNAME_BIO_QUERY: &str =
    "UPDATE voicesphere.users SET username = ?, bio = ?, updated_at = toTimestamp(now()) WHERE id = ?";
const UPDATE_USERNAME_ONLY_QUERY: &str =
    "UPDATE voicesphere.users SET username = ?, updated_at = toTimestamp(now()) WHERE id = ?";
const DELETE_USER_QUERY: &str = "DELETE FROM voicesphere.users WHERE id = ?";

pub struct UserDB;

impl UserDB {
    pub async fn create_user(session: &Arc<Session>, user: &User) -> Result<bool, String> {
        log::info!("Creating user with id={}", user.id);
        let id = &user.id;

        let prepared_query = session
            .prepare(CREATE_USER_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare create user query: {}", e))?;

        match session.execute_unpaged(&prepared_query, &user).await {
            Ok(_) => {
                log::info!("User created successfully: id={}", id);
                Ok(true)
            }
            Err(e) => {
                log::error!("Failed to create user: id={}, error={}", id, e);
                Err(format!("Failed to create user: {}", e))
            }
        }
    }

    pub async fn get_user(session: &Arc<Session>, id: &str) -> Result<Option<User>, String> {
        log::info!("Fetching user data for id={}", id);

        let prepared_query = session
            .prepare(GET_USER_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare get user query: {}", e))?;

        let rows = session
            .execute_unpaged(&prepared_query, (id,))
            .await
            .map_err(|e| format!("Failed to execute get user query: {}", e))?;

        let row_result = rows
            .into_rows_result()
            .map_err(|e| format!("Failed to convert rows result: {}", e))?;

        let mut users_rows = row_result
            .rows::<User>()
            .map_err(|e| format!("Failed to map rows to User struct: {}", e))?;

        match users_rows.next().transpose() {
            Ok(Some(user)) => {
                log::debug!("User found: id={}", id);
                Ok(Some(user))
            }
            Ok(None) => {
                log::warn!("User not found: id={}", id);
                Ok(None)
            }
            Err(e) => {
                log::error!("Failed to deserialize user data: {}", e);
                Err(format!("Failed to map user data: {}", e))
            }
        }
    }

    pub async fn user_exists(session: &Arc<Session>, id: &str) -> Result<Option<User>, String> {
        log::info!("Checking if user exists: id={}", id);
        Self::get_user(session, id).await
    }

    pub async fn get_user_stats(
        session: &Arc<Session>,
        user_id: &str,
    ) -> Result<Option<UserStatsData>, String> {
        let stmt = session
            .prepare(GET_USER_STATS_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare get user stats query: {}", e))?;

        let rows = session
            .execute_unpaged(&stmt, (user_id,))
            .await
            .map_err(|e| format!("Failed to execute get user stats query: {}", e))?;

        let row_result = rows
            .into_rows_result()
            .map_err(|e| format!("Failed to convert user stats result: {}", e))?;

        match row_result
            .rows::<UserStatsRow>()
            .map_err(|e| format!("Failed to deserialize user stats: {}", e))?
            .next()
            .transpose()
            .map_err(|e| format!("Failed to parse user stats row: {}", e))?
        {
            None => Ok(None),
            Some(row) => Ok(Some(UserStatsData {
                user_id: row.user_id,
                topic_count: row.topic_count,
                call_count: row.call_count,
                likes_given: row.likes_given,
                likes_received: row.likes_received,
                comments_given: row.comments_given,
                comments_received: row.comments_received,
                followers_count: row.followers_count,
                following_count: row.following_count,
            })),
        }
    }

    pub async fn update_last_login(
        session: &Arc<Session>,
        id: &str,
        last_login_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, String> {
        log::info!("Updating last login time for user id={}", id);
        let prepare_query = session
            .prepare(UPDATE_LAST_LOGIN_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare update last login query: {}", e))?;

        session
            .execute_unpaged(&prepare_query, (last_login_at, id))
            .await
            .map_err(|e| format!("Failed to update last login time: {}", e))?;

        log::info!("Last login time updated for user id={}", id);
        Ok(true)
    }

    pub async fn update_user(
        session: &Arc<Session>,
        id: &str,
        username: &str,
        bio: Option<&str>,
    ) -> Result<(), String> {
        match bio {
            Some(b) => {
                let stmt = session
                    .prepare(UPDATE_USERNAME_BIO_QUERY)
                    .await
                    .map_err(|e| format!("Failed to prepare update user query: {}", e))?;
                session
                    .execute_unpaged(&stmt, (username, b, id))
                    .await
                    .map_err(|e| format!("Failed to update user: {}", e))?;
            }
            None => {
                let stmt = session
                    .prepare(UPDATE_USERNAME_ONLY_QUERY)
                    .await
                    .map_err(|e| format!("Failed to prepare update user query: {}", e))?;
                session
                    .execute_unpaged(&stmt, (username, id))
                    .await
                    .map_err(|e| format!("Failed to update user: {}", e))?;
            }
        }
        log::info!("User updated: id={}", id);
        Ok(())
    }

    pub async fn delete_user(session: &Arc<Session>, id: &str) -> Result<bool, String> {
        let prepared_query = session
            .prepare(DELETE_USER_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare delete user query: {}", e))?;

        session
            .execute_unpaged(&prepared_query, (id,))
            .await
            .map_err(|e| format!("Failed to delete user: {}", e))?;

        log::info!("User deleted: id={}", id);
        Ok(true)
    }
}
