use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Request, StatusCode},
    response::IntoResponse,
};
use jsonwebtoken::{encode, jwk::JwkSet, Algorithm, EncodingKey, Header};
use kernel::AppError;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    oidc::decode_with_jwks, role_has_permission, AuthUser, Permission, Role,
};

const KEY_ID: &str = "identity-test-key";
const ISSUER: &str = "http://keycloak.test/realms/sensefoundry";
const TEST_PRIVATE_KEY: &[u8] = br#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDJETqse41HRBsc
7cfcq3ak4oZWFCoZlcic525A3FfO4qW9BMtRO/iXiyCCHn8JhiL9y8j5JdVP2Q9Z
IpfElcFd3/guS9w+5RqQGgCR+H56IVUyHZWtTJbKPcwWXQdNUX0rBFcsBzCRESJL
eelOEdHIjG7LRkx5l/FUvlqsyHDVJEQsHwegZ8b8C0fz0EgT2MMEdn10t6Ur1rXz
jMB/wvCg8vG8lvciXmedyo9xJ8oMOh0wUEgxziVDMMovmC+aJctcHUAYubwoGN8T
yzcvnGqL7JSh36Pwy28iPzXZ2RLhAyJFU39vLaHdljwthUaupldlNyCfa6Ofy4qN
ctlUPlN1AgMBAAECggEAdESTQjQ70O8QIp1ZSkCYXeZjuhj081CK7jhhp/4ChK7J
GlFQZMwiBze7d6K84TwAtfQGZhQ7km25E1kOm+3hIDCoKdVSKch/oL54f/BK6sKl
qlIzQEAenho4DuKCm3I4yAw9gEc0DV70DuMTR0LEpYyXcNJY3KNBOTjN5EYQAR9s
2MeurpgK2MdJlIuZaIbzSGd+diiz2E6vkmcufJLtmYUT/k/ddWvEtz+1DnO6bRHh
xuuDMeJA/lGB/EYloSLtdyCF6sII6C6slJJtgfb0bPy7l8VtL5iDyz46IKyzdyzW
tKAn394dm7MYR1RlUBEfqFUyNK7C+pVMVoTwCC2V4QKBgQD64syfiQ2oeUlLYDm4
CcKSP3RnES02bcTyEDFSuGyyS1jldI4A8GXHJ/lG5EYgiYa1RUivge4lJrlNfjyf
dV230xgKms7+JiXqag1FI+3mqjAgg4mYiNjaao8N8O3/PD59wMPeWYImsWXNyeHS
55rUKiHERtCcvdzKl4u35ZtTqQKBgQDNKnX2bVqOJ4WSqCgHRhOm386ugPHfy+8j
m6cicmUR46ND6ggBB03bCnEG9OtGisxTo/TuYVRu3WP4KjoJs2LD5fwdwJqpgtHl
yVsk45Y1Hfo+7M6lAuR8rzCi6kHHNb0HyBmZjysHWZsn79ZM+sQnLpgaYgQGRbKV
DZWlbw7g7QKBgQCl1u+98UGXAP1jFutwbPsx40IVszP4y5ypCe0gqgon3UiY/G+1
zTLp79GGe/SjI2VpQ7AlW7TI2A0bXXvDSDi3/5Dfya9ULnFXv9yfvH1QwWToySpW
Kvd1gYSoiX84/WCtjZOr0e0HmLIb0vw0hqZA4szJSqoxQgvF22EfIWaIaQKBgQCf
34+OmMYw8fEvSCPxDxVvOwW2i7pvV14hFEDYIeZKW2W1HWBhVMzBfFB5SE8yaCQy
pRfOzj9aKOCm2FjjiErVNpkQoi6jGtLvScnhZAt/lr2TXTrl8OwVkPrIaN0bG/AS
aUYxmBPCpXu3UjhfQiWqFq/mFyzlqlgvuCc9g95HPQKBgAscKP8mLxdKwOgX8yFW
GcZ0izY/30012ajdHY+/QK5lsMoxTnn0skdS+spLxaS5ZEO4qvPVb8RAoCkWMMal
2pOhmquJQVDPDLuZHdrIiKiDM20dy9sMfHygWcZjQ4WSxf/J7T9canLZIXFhHAZT
3wc9h4G8BBCtWN2TN/LsGZdB
-----END PRIVATE KEY-----"#;

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    exp: usize,
    iss: String,
    email: String,
    realm_access: TestRealmAccess,
}

#[derive(Serialize)]
struct TestRealmAccess {
    roles: Vec<String>,
}

fn test_jwks() -> JwkSet {
    serde_json::from_value(serde_json::json!({
        "keys": [{
            "kty": "RSA",
            "kid": KEY_ID,
            "use": "sig",
            "alg": "RS256",
            "n": "yRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5_CYYi_cvI-SXVT9kPWSKXxJXBXd_4LkvcPuUakBoAkfh-eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG_AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi-yUod-j8MtvIj812dkS4QMiRVN_by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQ",
            "e": "AQAB"
        }]
    }))
    .expect("test JWKS must be valid")
}

fn token(expires_at: usize, roles: &[&str]) -> (String, Uuid) {
    let user_id = Uuid::new_v4();
    let claims = TestClaims {
        sub: user_id.to_string(),
        exp: expires_at,
        iss: ISSUER.into(),
        email: "editor@test.local".into(),
        realm_access: TestRealmAccess {
            roles: roles.iter().map(|role| (*role).to_owned()).collect(),
        },
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KEY_ID.into());
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY).expect("test RSA key must be valid"),
    )
    .expect("test token must encode");
    (token, user_id)
}

fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must follow Unix epoch")
        .as_secs() as usize
}

#[test]
fn role_from_str_maps_all_13_roles() {
    let names = [
        "user",
        "contributor",
        "corpus_admin",
        "editor",
        "linguist",
        "arbiter",
        "model_engineer",
        "data_governance",
        "legal",
        "security_admin",
        "publication_manager",
        "ops",
        "project_lead",
    ];

    for name in names {
        assert!(Role::from_str(name).is_some(), "role not found: {name}");
    }
}

#[test]
fn unknown_role_returns_none() {
    assert!(Role::from_str("superadmin").is_none());
}

#[tokio::test]
async fn mock_rs256_token_decodes_without_keycloak() {
    let (token, user_id) = token(now() + 300, &["editor", "user"]);

    let claims = decode_with_jwks(&token, &test_jwks(), ISSUER).expect("token is valid");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.email.as_deref(), Some("editor@test.local"));
    assert_eq!(
        claims.realm_access.expect("roles are present").roles,
        ["editor", "user"]
    );
}

#[test]
fn expired_mock_token_is_unauthorized() {
    let (token, _) = token(now() - 120, &["editor"]);

    let result = decode_with_jwks(&token, &test_jwks(), ISSUER);

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn missing_bearer_returns_unauthorized() {
    let (mut parts, _) = Request::new(Body::empty()).into_parts();

    let result = AuthUser::from_request_parts(&mut parts, &()).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[test]
fn role_deny_returns_forbidden() {
    let user = AuthUser {
        user_id: Uuid::new_v4(),
        external_id: "keycloak-user".into(),
        email: "user@test.local".into(),
        roles: vec![Role::User],
    };

    let error = user
        .require_any_role(&[Role::Editor])
        .expect_err("user role must not grant editor access");

    assert_eq!(error.into_response().status(), StatusCode::FORBIDDEN);
    assert!(!role_has_permission(
        Role::Editor,
        Permission::ManageUserRoles
    ));
}
