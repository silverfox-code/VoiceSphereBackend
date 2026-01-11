use chrono::{Duration, Utc};
use http::header;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation, decode, decode_header, encode, errors::Error, jwk::JwkSet};
use serde::{Deserialize, Serialize};

use crate::User;

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

pub fn generate_jwt_token(user : &User, secret: &str) -> Result<String, Error> {

    let expiration = Utc::now()
    .checked_add_signed(Duration::days(30)).unwrap();

    let mut new_user = user.clone();
    // new_user.expires_at = expiration; 

    let header = jsonwebtoken::Header::new(Algorithm::HS256);
    let claims = new_user;
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    let token = encode(&header, &claims, &encoding_key);
    return token;
}