use chrono::{DateTime, Utc};
use scylla::frame::value::CqlTimestamp;
use scylla::{FromRow, SerializeRow};

// User model
use serde::{Deserialize, Serialize};

use crate::authenticate::GoogleClaims;

#[derive(SerializeRow, FromRow, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub bio: Option<String>,
    pub profile_picture: Option<String>,
}
