// Import necessary modules and traits from external crates `jsonwebtoken`, `serde`, `std`, and `uuid`.
// These are used for JWT creation/validation, serialization/deserialization, environment variable handling, and UUID generation.
//! This module implements JWT-based authentication utilities.
//! It defines the structure of our token claims and provides functions
//! to create and validate JSON Web Tokens using the HS256 algorithm.

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::env;
use uuid::Uuid;
use chrono::{Utc, Duration};

/// The set of data we encode into each JWT.
///
/// - `sub`: Subject, used here to store the user’s UUID as a string.
/// - `exp`: Expiration timestamp, expressed as seconds since the epoch.
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

/// Generate a signed JWT for the specified `user_id`.
///
/// Workflow:
/// 1. Load the secret key from `JWT_SECRET` env var (default: "secretkey").
/// 2. Compute token expiration 1 hour from current UTC time.
/// 3. Construct `Claims` struct and encode with HS256.
/// 4. Panic only if encoding unexpectedly fails.

pub fn create_token(user_id: Uuid) -> String {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secretkey".to_string());
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .expect("Failed to compute expiration timestamp")
        .timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Token creation failed")
}

/// Validate the JWT and return the user ID (`sub` claim) on success.
/// Returns `None` if token is invalid or expired.

pub fn validate_token(token: &str) -> Option<String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secretkey".to_string());
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims.sub)
}