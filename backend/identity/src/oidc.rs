use std::env;

use anyhow::Context;
use jsonwebtoken::{
    decode, decode_header,
    jwk::JwkSet,
    Algorithm, DecodingKey, Validation,
};
use kernel::{AppError, AppResult};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub exp: usize,
    pub iss: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

pub async fn validate_jwt(token: &str, jwks_url: &str) -> AppResult<JwtClaims> {
    let jwks = reqwest::get(jwks_url)
        .await
        .context("failed to fetch Keycloak JWKS")?
        .error_for_status()
        .context("Keycloak JWKS endpoint returned an error")?
        .json::<JwkSet>()
        .await
        .context("failed to decode Keycloak JWKS")?;

    decode_with_jwks(token, &jwks, &configured_issuer())
}

pub(crate) fn decode_with_jwks(
    token: &str,
    jwks: &JwkSet,
    issuer: &str,
) -> AppResult<JwtClaims> {
    let header = decode_header(token).map_err(|_| AppError::Unauthorized)?;
    if header.alg != Algorithm::RS256 {
        return Err(AppError::Unauthorized);
    }
    let kid = header.kid.ok_or(AppError::Unauthorized)?;
    let jwk = jwks.find(&kid).ok_or(AppError::Unauthorized)?;
    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|_| AppError::Unauthorized)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer]);
    validation.set_required_spec_claims(&["exp", "iss", "sub"]);
    validation.validate_exp = true;

    decode::<JwtClaims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
}

fn configured_issuer() -> String {
    env::var("KEYCLOAK_ISSUER")
        .unwrap_or_else(|_| "http://keycloak:8080/realms/sensefoundry".into())
}
