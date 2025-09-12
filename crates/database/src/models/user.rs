use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, query_as};
use uuid::Uuid;

use crate::db;
use crate::utils::error::{DatabaseError, ModelResult};

#[derive(FromRow)]
pub struct UserModel {
    id: Uuid,
    username: String,
    level: i32,
    xp: i32,
    email: String,
    created_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct UserCreation {
    username: String,
    email: String,
}

#[derive(serde::Deserialize)]
pub struct UserUpdate {
    username: Option<String>,
    level: Option<i32>,
    xp: Option<i32>,
}

#[derive(Serialize)]
pub struct UserResult {
    id: String,
    level: i32,
    xp: i32,
    username: String,
    created_at: NaiveDateTime,
}

impl UserModel {
    pub async fn create_new(creation: UserCreation) -> ModelResult<Self> {
        let user = query_as!(
            Self,
            r#"
                INSERT INTO users (
                    username,
                    email
                )
                VALUES ($1, $2)
                RETURNING *
            "#,
            creation.username,
            creation.email,
        )
        .fetch_one(db!())
        .await?;

        Ok(user)
    }

    pub async fn get(id: String) -> ModelResult<Self> {
        let user = query_as!(
            Self,
            r#"
                SELECT *
                FROM users
                WHERE
                    id = $1
            "#,
            Uuid::parse_str(&id)?
        )
        .fetch_one(db!())
        .await?;

        Ok(user)
    }

    pub async fn edit(&self, update: UserUpdate) -> ModelResult<Self> {
        query_as!(
            Self,
            r#"
                UPDATE users
                SET
                    username = COALESCE($1, username),
                    level = COALESCE($2, level),
                    xp = COALESCE($3, xp)
                WHERE
                    id = $4
                RETURNING *
            "#,
            update.username,
            update.level,
            update.xp,
            self.id
        )
        .fetch_optional(db!())
        .await?
        .ok_or(DatabaseError::ModelNotFound("user"))
    }

    pub fn to_result(&self) -> UserResult {
        UserResult {
            id: self.id.into(),
            username: self.username.clone(),
            level: self.level,
            xp: self.xp,
            created_at: self.created_at,
        }
    }
}
