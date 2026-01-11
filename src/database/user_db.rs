use chrono;
// User database operations
use scylla::{FromRow, IntoTypedRows, Session};
use std::sync::Arc;

use crate::{handlers::user, User};

// Query constants for production use
const CREATE_USER_QUERY: &str = "INSERT INTO voicesphere.users (
    id,
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
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,?)";

const GET_USER_QUERY: &str = "SELECT
    id,
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

const UPDATE_LAST_LOGIN_QUERY: &str = "UPDATE voicesphere.users SET last_login_at = ? WHERE id = ?";
const UPDATE_USERNAME_BIO_QUERY: &str =
    "UPDATE voicesphere.users SET username = ?, bio = ?, updated_at = now() WHERE id = ?";
const UPDATE_USERNAME_ONLY_QUERY: &str =
    "UPDATE voicesphere.users SET username = ?, updated_at = now() WHERE id = ?";
const UPDATE_BIO_ONLY_QUERY: &str =
    "UPDATE voicesphere.users SET bio = ?, updated_at = now() WHERE id = ?";
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
            .map_err(|e| format!("Failed to parse user data: {}", e))?;

        match rows.first_row_typed::<User>() {
            Ok(user) => {
                log::debug!("User found: id={}", id);
                Ok(Some(user))
            }
            Err(e) => {
                if e.to_string().contains("empty") {
                    log::warn!("User not found: id={}", id);
                    Ok(None)
                } else {
                    log::error!("Failed to deserialize user data: {}", e);
                    Err(format!("Failed to map user data: {}", e))
                }
            }
        }
    }

    pub async fn user_exists(session: &Arc<Session>, id: &str) -> Result<Option<User>, String> {
        log::info!("Checking if user exists: id={}", id);
        self::UserDB::get_user(session, id).await
    }

    pub async fn update_last_login(session: &Arc<Session>, id: &str, last_login_at: chrono::DateTime<chrono::Utc>) -> Result<bool, String> {
        log::info!("Updating last login time for user id={}", id);
        let prepare_query = session
            .prepare(UPDATE_LAST_LOGIN_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare update last time query: {}", e))?;

        match session
            .execute_unpaged(&prepare_query, (last_login_at, id))
            .await
        {
            Ok(_) => {
                log::info!("Last login time updated successfully for user id={}", id);
                Ok(true)
            }
            Err(e) => {
                log::error!(
                    "Failed to update last login time for user id={}, error={}",
                    id,
                    e
                );
                Err(format!("Failed to update last login time: {}", e))
            }
        }
    }

    pub async fn update_user(
        session: &Arc<Session>,
        id: &str,
        username: Option<&str>,
        bio: Option<&str>,
    ) -> Result<bool, String> {
        // Validate that at least one field is provided
        if username.is_none() && bio.is_none() {
            log::warn!("Update user called with no fields to update: id={}", id);
            return Ok(false);
        }

        // Check if user exists first
        let user_exists = self::UserDB::user_exists(session, id).await?;
        if user_exists.is_none() {
            return Err(format!("User not found: id={}", id));
        }

        return Ok(true);
    }

    pub async fn delete_user(session: &Arc<Session>, id: &str) -> Result<bool, String> {
        let prepared_query = session
            .prepare(DELETE_USER_QUERY)
            .await
            .map_err(|e| format!("Failed to prepare delete user query: {}", e))?;

        match session.execute_unpaged(&prepared_query, (id,)).await {
            Ok(_) => {
                log::info!("User deleted successfully: id={}", id);
                Ok(true)
            }
            Err(e) => {
                log::error!("Failed to delete user: id={}, error={}", id, e);
                Err(format!("Failed to delete user: {}", e))
            }
        }
    }

    pub async fn search_users(
        session: &Arc<Session>,
        _query: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        // TODO: Implement full-text search with secondary indexes or Solr integration
        log::warn!("Search users not yet implemented");
        Ok(vec![])
    }
}
