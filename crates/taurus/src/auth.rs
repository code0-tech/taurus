//! Mints the short-lived JWT Taurus presents to Aquila's gRPC services.
//!
//! Aquila's service configuration pre-provisions a secret per identity
//! (still called `token` on both sides for historical reasons); Taurus signs
//! an HS256 JWT with that secret instead of sending it as a plain bearer
//! credential, so Aquila can verify the request without the secret itself
//! ever crossing the wire.

use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;

/// The identity every `taurus-*` instance authenticates as - matches the
/// `taurus` family identifier Aquila's service configuration registers a
/// runtime secret under (see `ServiceConfiguration::extract_service_name`).
const SUBJECT: &str = "taurus";

/// How long a minted JWT stays valid. Short-lived since a fresh one is
/// minted for every outgoing request rather than reused across a
/// long-lived connection.
const TTL_SECONDS: u64 = 60;

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: u64,
}

/// Encodes a fresh JWT authenticating Taurus to Aquila, signed with
/// `aquila_token` (the shared secret from `Config::aquila_token`).
pub fn aquila_jwt(aquila_token: &str) -> String {
    let claims = Claims {
        sub: SUBJECT.to_string(),
        exp: now_unix_seconds().saturating_add(TTL_SECONDS),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(aquila_token.as_bytes()),
    )
    .unwrap_or_else(|error| panic!("failed to encode Aquila authentication JWT: {error}"))
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, Validation, decode};

    #[derive(serde::Deserialize)]
    struct DecodedClaims {
        sub: String,
        exp: u64,
    }

    #[test]
    fn aquila_jwt_is_signed_with_the_given_secret_and_carries_the_taurus_subject() {
        let token = aquila_jwt("shared-secret");

        let data = decode::<DecodedClaims>(
            &token,
            &DecodingKey::from_secret(b"shared-secret"),
            &Validation::new(Algorithm::HS256),
        )
        .unwrap();

        assert_eq!(data.claims.sub, "taurus");
        assert!(data.claims.exp > now_unix_seconds());
    }

    #[test]
    fn aquila_jwt_does_not_verify_against_a_different_secret() {
        let token = aquila_jwt("shared-secret");

        let result = decode::<DecodedClaims>(
            &token,
            &DecodingKey::from_secret(b"wrong-secret"),
            &Validation::new(Algorithm::HS256),
        );

        assert!(result.is_err());
    }
}
