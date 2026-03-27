use chrono::{DateTime, Utc};
use scylla::{DeserializeRow, SerializeRow};

// User model
use serde::{Deserialize, Serialize};

use crate::authenticate::GoogleClaims;

#[derive(SerializeRow, Debug, Clone, Serialize, Deserialize, DeserializeRow)]
pub struct User {
    // Identity
    pub id: String,
    pub device_id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub email_verified: bool,

    // Profile
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub language: String,
    pub timezone: String,

    // Status
    pub is_active: bool,
    pub is_shadow_banned: bool,
    pub report_count: i32,

    // Auth control
    pub session_version: i32,
    pub last_login_at: DateTime<Utc>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn create_user_from_google_claims(claims: GoogleClaims, device_id: String) -> Self {
        Self {
            id: claims.sub,
            device_id: device_id.to_string(),
            username: claims.name.to_string(),
            display_name: claims.name.to_string(),
            email: claims.email.to_string(),
            email_verified: claims.email_verified,
            bio: None,
            avatar_url: None,
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            is_active: true,
            is_shadow_banned: false,
            report_count: 0,
            session_version: 0,
            last_login_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

/// Safe public view of a user — no sensitive fields exposed via API.
#[derive(Debug, Serialize, Clone)]
pub struct ProfileData {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub language: String,
    pub is_active: bool,
}

impl From<User> for ProfileData {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            bio: u.bio,
            avatar_url: u.avatar_url,
            language: u.language,
            is_active: u.is_active,
        }
    }
}

/// User activity stats from the user_stats COUNTER table.
#[derive(Debug, Serialize, Clone)]
pub struct UserStatsData {
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

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: String, // always required — username must never be empty
    pub bio: Option<String>,
}
