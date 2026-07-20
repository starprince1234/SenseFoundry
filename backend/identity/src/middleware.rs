use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use kernel::AppError;
use serde::Serialize;
use uuid::Uuid;

use crate::{validate_jwt, Role};

const DEFAULT_JWKS_URL: &str =
    "http://keycloak:8080/realms/sensefoundry/protocol/openid-connect/certs";

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub external_id: String,
    pub email: String,
    pub roles: Vec<Role>,
}

impl AuthUser {
    pub fn has_role(&self, required: Role) -> bool {
        self.roles
            .iter()
            .copied()
            .any(|role| role.satisfies(required))
    }

    pub fn require_any_role(&self, required: &[Role]) -> Result<(), AppError> {
        if required.iter().copied().any(|role| self.has_role(role)) {
            Ok(())
        } else {
            Err(AppError::Forbidden("required role is missing".into()))
        }
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|value| !value.is_empty())
            .ok_or(AppError::Unauthorized)?;
        let jwks_url = std::env::var("KEYCLOAK_JWKS_URL")
            .unwrap_or_else(|_| DEFAULT_JWKS_URL.into());
        let claims = validate_jwt(token, &jwks_url).await?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
        let roles = claims
            .realm_access
            .map(|access| {
                access
                    .roles
                    .iter()
                    .filter_map(|role| Role::from_str(role))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            user_id,
            external_id: claims.sub,
            email: claims.email.unwrap_or_default(),
            roles,
        })
    }
}
