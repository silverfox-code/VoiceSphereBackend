use chrono::{Duration, Utc};
use http::header;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation, decode, decode_header, encode, errors::Error, jwk::JwkSet};
use serde::{Deserialize, Serialize};

use crate::{AppError, User};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleClaims {
    pub sub: String,
    pub email: String,
    pub email_verified: bool, // if not verified -> either reject or notify user to verify
    pub name: String,
}

pub async fn verify_google_token(token: &str, client_id: &str) -> Result<GoogleClaims, String> {
    // fetch Google's public keys
    let jwks: JwkSet = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
        .await
        .map_err(|e| format!("Failed to fetch JWKs: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKs: {}", e))?;

    log::info!("Fetched JWKs: {:?}", jwks);

    log::info!("=== AUDIENCE MISMATCH DEBUG ===");
    log::info!("Expected aud:  {}", client_id);

    //decode
    let header =
        decode_header(token).map_err(|e| format!("Failed to decode token header: {}", e))?;
    let kid = header.kid.ok_or("No 'kid' found in token header")?;

    log::info!("Token kid: {}", kid);
    //find matching key
    let jwk = jwks
        .keys
        .iter()
        .find(|jwk_key| jwk_key.common.key_id.as_ref() == Some(&kid))
        .ok_or("No matching JWK found")?;

    //decode token
    let decoding_key =
        DecodingKey::from_jwk(jwk).map_err(|e| format!("Failed to create decoding key: {}", e))?;
    let mut validation = Validation::new(Algorithm::RS256);

    validation.set_audience(&[client_id]);
    validation.set_issuer(&[
        "https://accounts.google.com".to_string(),
        "accounts.google.com".to_string(),
    ]);

    let token_data = decode::<GoogleClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Failed to decode token: {}", e))?;

    log::info!("Decoded token data: {:?}", token_data);
    Ok(token_data.claims)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub device_id: String,
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
    pub report_count: i32,

    // Auth control
    pub session_version: i32,
    pub exp: i64
}

pub fn generate_jwt_token(user: &User, secret: &str) -> Result<(String, i64), Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(30))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.clone(),
        device_id: user.device_id.clone(),
        display_name: user.display_name.clone(),
        email: user.email.clone(),
        email_verified: user.email_verified,
        bio: user.bio.clone(),
        avatar_url: user.avatar_url.clone(),
        language: user.language.clone(),
        timezone: user.timezone.clone(),
        is_active: user.is_active,
        report_count: user.report_count,
        session_version: user.session_version,
        exp: expiration,
    };

    let header = jsonwebtoken::Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    let token = encode(&header, &claims, &encoding_key)?;
    Ok((token, expiration))
}

pub fn verify_refresh_token(token: &str, device_id: &str, secret: &str) -> Result<String, AppError> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
    .map_err(|e| AppError::Unauthorized(format!("Failed to decode token: {}", e)))?;   
    
    if token_data.claims.device_id != device_id {
        return Err(AppError::Unauthorized("Device ID mismatch".to_string()));
    }

    if token_data.claims.exp < Utc::now().timestamp() {
        return Err(AppError::Unauthorized("Token expired".to_string()));
    } 

    if token_data.claims.session_version != token_data.claims.session_version {
        return Err(AppError::Unauthorized("Session version mismatch".to_string()));
    }
    
    // let user = User{
    //     id: token_data.claims.sub,
    //     device_id: token_data.claims.device_id,
    //     display_name: token_data.claims.display_name,
    //     email: token_data.claims.email,
    //     email_verified: token_data.claims.email_verified,
    //     bio: token_data.claims.bio,
    //     avatar_url: token_data.claims.avatar_url,
    //     language: token_data.claims.language,
    //     timezone: token_data.claims.timezone,
    //     is_active: token_data.claims.is_active,
    //     report_count: token_data.claims.report_count,
    //     session_version: token_data.claims.session_version,
    //     last_login_at: token_data.claims.exp,
    //     created_at: token_data.claims.exp,
    //     updated_at: token_data.claims.exp,
    // }   
    // log::info!("User Authenticated {:?}", user);
    // Ok(user)

    Ok("User Authenticated".to_string())
}