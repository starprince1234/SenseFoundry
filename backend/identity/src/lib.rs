pub mod middleware;
pub mod oidc;
pub mod rbac;
pub mod routes;

pub use middleware::AuthUser;
pub use oidc::{validate_jwt, JwtClaims, RealmAccess};
pub use rbac::{has_permission, role_has_permission, Permission, Role};
pub use routes::{router, UpdateUserRoles, User};

#[cfg(test)]
mod tests;
