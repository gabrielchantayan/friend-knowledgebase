//! # User-Friend Relationship Repository
//!
//! Repository for user-friend relationship database operations.
//! These track how the user personally knows each friend.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::UserFriendRelationship;

use super::base::{Repository, RepositoryContext};
use super::error::RepositoryError;

/// Input for creating a new user-friend relationship.
pub struct CreateUserFriendRelationshipInput {
    /// The friend this relationship is for
    pub friend_id: Uuid,
    /// How the user knows this friend (e.g., "coworker", "neighbor")
    pub relationship_type: String,
}

/// Input for updating an existing user-friend relationship.
pub struct UpdateUserFriendRelationshipInput {
    pub relationship_type: Option<String>,
}

/// Repository for user-friend relationship database operations.
pub struct UserFriendRelationshipRepository {
    ctx: RepositoryContext,
}

impl UserFriendRelationshipRepository {
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    /// List all user-friend relationships for a specific friend.
    ///
    /// A friend can have multiple relationship types (e.g., both
    /// "coworker" and "neighbor" if you work with a neighbor).
    ///
    /// # Arguments
    ///
    /// * `friend_id` - The UUID of the friend
    pub async fn list_by_friend(
        &self,
        friend_id: Uuid,
    ) -> Result<Vec<UserFriendRelationship>, RepositoryError> {
        let relationships = sqlx::query_as!(
            UserFriendRelationship,
            r#"
            SELECT id, friend_id, relationship_type, created_at, updated_at
            FROM user_friend_relationships
            WHERE friend_id = $1
            ORDER BY relationship_type ASC
            "#,
            friend_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationships)
    }

    /// Find a relationship by friend and type.
    ///
    /// Useful for checking if a specific relationship type already exists.
    ///
    /// # Arguments
    ///
    /// * `friend_id` - The UUID of the friend
    /// * `relationship_type` - The type to search for
    pub async fn find_by_friend_and_type(
        &self,
        friend_id: Uuid,
        relationship_type: &str,
    ) -> Result<Option<UserFriendRelationship>, RepositoryError> {
        let relationship = sqlx::query_as!(
            UserFriendRelationship,
            r#"
            SELECT id, friend_id, relationship_type, created_at, updated_at
            FROM user_friend_relationships
            WHERE friend_id = $1 AND relationship_type = $2
            "#,
            friend_id,
            relationship_type
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationship)
    }
}

#[async_trait]
impl Repository for UserFriendRelationshipRepository {
    type Entity = UserFriendRelationship;
    type CreateInput = CreateUserFriendRelationshipInput;
    type UpdateInput = UpdateUserFriendRelationshipInput;

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<UserFriendRelationship>, RepositoryError> {
        let relationship = sqlx::query_as!(
            UserFriendRelationship,
            r#"
            SELECT id, friend_id, relationship_type, created_at, updated_at
            FROM user_friend_relationships
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationship)
    }

    async fn create(
        &self,
        input: CreateUserFriendRelationshipInput,
    ) -> Result<UserFriendRelationship, RepositoryError> {
        let relationship = sqlx::query_as!(
            UserFriendRelationship,
            r#"
            INSERT INTO user_friend_relationships (friend_id, relationship_type)
            VALUES ($1, $2)
            RETURNING id, friend_id, relationship_type, created_at, updated_at
            "#,
            input.friend_id,
            input.relationship_type
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationship)
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateUserFriendRelationshipInput,
    ) -> Result<UserFriendRelationship, RepositoryError> {
        let relationship = sqlx::query_as!(
            UserFriendRelationship,
            r#"
            UPDATE user_friend_relationships
            SET relationship_type = COALESCE($2, relationship_type)
            WHERE id = $1
            RETURNING id, friend_id, relationship_type, created_at, updated_at
            "#,
            id,
            input.relationship_type
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(relationship)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_friend_relationships
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(result.rows_affected() > 0)
    }
}
