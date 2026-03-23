use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, query_as};
use uuid::Uuid;

use crate::db;
use crate::utils::error::{DatabaseError, ModelResult};

#[derive(FromRow)]
pub struct CommunityModel {
    id: Uuid,
    owner_id: Uuid,
    name: String,
    width: i32,
    height: i32,
    is_public: bool,
    billing_plan: String,
    created_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct CommunityCreation {
    #[serde(default)]
    pub owner_id: String,
    pub name: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub is_public: Option<bool>,
    pub billing_plan: Option<String>,
}

#[derive(Serialize)]
pub struct CommunityResult {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub is_public: bool,
    pub billing_plan: String,
    pub created_at: NaiveDateTime,
}

#[derive(FromRow, Serialize)]
pub struct CommunityMembershipResult {
    pub community_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct MembershipCreation {
    pub community_id: String,
    pub user_id: String,
    pub role: Option<String>,
}

impl CommunityModel {
    pub async fn create_new(creation: CommunityCreation) -> ModelResult<Self> {
        let owner_id = Uuid::parse_str(&creation.owner_id)?;

        let community = query_as!(
            Self,
            r#"
                INSERT INTO communities (
                    owner_id,
                    name,
                    width,
                    height,
                    is_public,
                    billing_plan
                )
                VALUES ($1, $2, COALESCE($3, 512), COALESCE($4, 512), COALESCE($5, FALSE), COALESCE($6, 'free'))
                RETURNING *
            "#,
            owner_id,
            creation.name,
            creation.width,
            creation.height,
            creation.is_public,
            creation.billing_plan,
        )
        .fetch_one(db!())
        .await?;

        Ok(community)
    }

    pub async fn get_by_id(community_id: &str) -> ModelResult<Self> {
        let community = query_as!(
            Self,
            r#"
                SELECT *
                FROM communities
                WHERE id = $1
            "#,
            Uuid::parse_str(community_id)?
        )
        .fetch_optional(db!())
        .await?
        .ok_or(DatabaseError::ModelNotFound("community"))?;

        Ok(community)
    }

    pub async fn list_for_user(user_id: &str) -> ModelResult<Vec<Self>> {
        let communities = query_as!(
            Self,
            r#"
                SELECT c.*
                FROM communities c
                INNER JOIN community_members m ON m.community_id = c.id
                WHERE m.user_id = $1
                UNION
                SELECT *
                FROM communities c
                WHERE c.owner_id = $1
            "#,
            Uuid::parse_str(user_id)?
        )
        .fetch_all(db!())
        .await?;

        Ok(communities)
    }

    pub fn to_result(&self) -> CommunityResult {
        CommunityResult {
            id: self.id.to_string(),
            owner_id: self.owner_id.to_string(),
            name: self.name.clone(),
            width: self.width,
            height: self.height,
            is_public: self.is_public,
            billing_plan: self.billing_plan.clone(),
            created_at: self.created_at,
        }
    }
}

impl CommunityMembershipResult {
    pub async fn create_new(creation: MembershipCreation) -> ModelResult<Self> {
        let membership = query_as!(
            Self,
            r#"
                INSERT INTO community_members (
                    community_id,
                    user_id,
                    role
                )
                VALUES ($1, $2, COALESCE($3, 'member'))
                ON CONFLICT (community_id, user_id) DO UPDATE SET
                    role = COALESCE(EXCLUDED.role, community_members.role)
                RETURNING community_id::text, user_id::text, role, joined_at
            "#,
            Uuid::parse_str(&creation.community_id)?,
            Uuid::parse_str(&creation.user_id)?,
            creation.role
        )
        .fetch_one(db!())
        .await?;

        Ok(membership)
    }

    pub async fn list_for_community(community_id: &str) -> ModelResult<Vec<Self>> {
        let members = query_as!(
            Self,
            r#"
                SELECT community_id::text, user_id::text, role, joined_at
                FROM community_members
                WHERE community_id = $1
            "#,
            Uuid::parse_str(community_id)?
        )
        .fetch_all(db!())
        .await?;

        Ok(members)
    }

    pub async fn require_role(
        community_id: &str,
        user_id: &str,
        expected_roles: &[&str],
    ) -> ModelResult<()> {
        let membership = query_as!(
            Self,
            r#"
                SELECT community_id::text, user_id::text, role, joined_at
                FROM community_members
                WHERE community_id = $1 AND user_id = $2
            "#,
            Uuid::parse_str(community_id)?,
            Uuid::parse_str(user_id)?
        )
        .fetch_optional(db!())
        .await?;

        if let Some(member) = membership {
            if expected_roles.iter().any(|role| role.eq_ignore_ascii_case(&member.role)) {
                return Ok(());
            }
        }

        Err(DatabaseError::ModelNotFound("membership"))
    }
}
