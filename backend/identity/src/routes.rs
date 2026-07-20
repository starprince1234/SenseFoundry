use axum::{
    extract::{Path, Query, State},
    routing::{get, patch},
    Json, Router,
};
use chrono::{DateTime, Utc};
use kernel::{AppError, AppResult, Page, PageParams};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, PgPool, Row};
use uuid::Uuid;

use crate::{AuthUser, Role};

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub external_id: String,
    pub email: Option<String>,
    pub roles: Vec<Role>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoles {
    pub roles: Vec<Role>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/auth/me", get(current_user))
        .route("/users", get(list_users))
        .route("/users/:id", patch(update_user_roles))
        .with_state(pool)
}

async fn current_user(user: AuthUser) -> Json<AuthUser> {
    Json(user)
}

async fn list_users(
    State(pool): State<PgPool>,
    user: AuthUser,
    Query(params): Query<PageParams>,
) -> AppResult<Json<Page<User>>> {
    user.require_any_role(&[Role::CorpusAdmin, Role::SecurityAdmin, Role::ProjectLead])?;
    let rows = sqlx::query(
        "SELECT id, external_id, email, roles, created_at, updated_at \
         FROM users WHERE deleted_at IS NULL \
         ORDER BY created_at, id LIMIT $1 OFFSET $2",
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(&pool)
    .await?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL",
    )
    .fetch_one(&pool)
    .await?;
    let users = rows
        .into_iter()
        .map(user_from_row)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Json(Page::new(users, &params, Some(total))))
}

async fn update_user_roles(
    State(pool): State<PgPool>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(mut request): Json<UpdateUserRoles>,
) -> AppResult<Json<User>> {
    user.require_any_role(&[Role::SecurityAdmin, Role::ProjectLead])?;
    request.roles.sort_by_key(|role| role.as_str());
    request.roles.dedup();
    let role_names = serde_json::to_value(&request.roles)
        .map_err(|error| AppError::Internal(error.into()))?;
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        "UPDATE users SET roles = $1, updated_at = NOW() \
         WHERE id = $2 AND deleted_at IS NULL \
         RETURNING id, external_id, email, roles, created_at, updated_at",
    )
    .bind(role_names)
    .bind(id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("user {id}")))?;

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(id)
        .execute(&mut *transaction)
        .await?;
    for role in &request.roles {
        sqlx::query("INSERT INTO user_roles (user_id, role_name) VALUES ($1, $2)")
            .bind(id)
            .bind(role.as_str())
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;

    Ok(Json(user_from_row(row)?))
}

fn user_from_row(row: PgRow) -> AppResult<User> {
    let roles = serde_json::from_value(row.try_get("roles")?)
        .map_err(|error| AppError::Internal(error.into()))?;
    Ok(User {
        id: row.try_get("id")?,
        external_id: row.try_get("external_id")?,
        email: row.try_get("email")?,
        roles,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
