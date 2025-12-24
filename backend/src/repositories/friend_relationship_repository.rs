//! # Friend Relationship Repository
//!
//! Repository for friend-to-friend relationship database operations.
//! These track how friends know each other (e.g., siblings, coworkers).

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::FriendRelationship;

use super::base::{Repository, RepositoryContext};
use super::error::RepositoryError;

/// Input for creating a new friend relationship.
pub struct CreateFriendRelationshipInput {
    /// The user who owns both friends
    pub user_id: Uuid,
    /// First friend in the relationship
    pub friend_a_id: Uuid,
    /// Second friend in the relationship
    pub friend_b_id: Uuid,
    /// How A relates to B (e.g., "sibling of", "boss of")
    pub a_to_b: String,
    /// How B relates to A (optional, NULL means symmetric)
    pub b_to_a: Option<String>,
}

/// Input for updating an existing friend relationship.
pub struct UpdateFriendRelationshipInput {
    pub a_to_b: Option<String>,
    pub b_to_a: Option<String>,
}

/// Repository for friend relationship database operations.
pub struct FriendRelationshipRepository {
    ctx: RepositoryContext,
}

impl FriendRelationshipRepository {
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    /// List all relationships for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The UUID of the user
    pub async fn list_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRelationship>, RepositoryError> {
        let relationships = sqlx::query_as!(
            FriendRelationship,
            r#"
            SELECT id, user_id, friend_a_id, friend_b_id, a_to_b, b_to_a, created_at, updated_at
            FROM friend_relationships
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationships)
    }

    /// List all relationships involving a specific friend.
    ///
    /// This returns relationships where the friend is either friend_a or friend_b.
    ///
    /// # Arguments
    ///
    /// * `friend_id` - The UUID of the friend
    pub async fn list_by_friend(
        &self,
        friend_id: Uuid,
    ) -> Result<Vec<FriendRelationship>, RepositoryError> {
        let relationships = sqlx::query_as!(
            FriendRelationship,
            r#"
            SELECT id, user_id, friend_a_id, friend_b_id, a_to_b, b_to_a, created_at, updated_at
            FROM friend_relationships
            WHERE friend_a_id = $1 OR friend_b_id = $1
            ORDER BY created_at DESC
            "#,
            friend_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationships)
    }

    /// Find a relationship between two specific friends.
    ///
    /// # Note
    ///
    /// This checks both directions - the relationship could be stored as
    /// (A, B) or (B, A) in the database.
    pub async fn find_between(
        &self,
        friend_a_id: Uuid,
        friend_b_id: Uuid,
    ) -> Result<Option<FriendRelationship>, RepositoryError> {
        let relationship = sqlx::query_as!(
            FriendRelationship,
            r#"
            SELECT id, user_id, friend_a_id, friend_b_id, a_to_b, b_to_a, created_at, updated_at
            FROM friend_relationships
            WHERE (friend_a_id = $1 AND friend_b_id = $2)
               OR (friend_a_id = $2 AND friend_b_id = $1)
            "#,
            friend_a_id,
            friend_b_id
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationship)
    }
}

#[async_trait]
impl Repository for FriendRelationshipRepository {
    type Entity = FriendRelationship;
    type CreateInput = CreateFriendRelationshipInput;
    type UpdateInput = UpdateFriendRelationshipInput;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FriendRelationship>, RepositoryError> {
        let relationship = sqlx::query_as!(
            FriendRelationship,
            r#"
            SELECT id, user_id, friend_a_id, friend_b_id, a_to_b, b_to_a, created_at, updated_at
            FROM friend_relationships
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
        input: CreateFriendRelationshipInput,
    ) -> Result<FriendRelationship, RepositoryError> {
        let relationship = sqlx::query_as!(
            FriendRelationship,
            r#"
            INSERT INTO friend_relationships (user_id, friend_a_id, friend_b_id, a_to_b, b_to_a)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, friend_a_id, friend_b_id, a_to_b, b_to_a, created_at, updated_at
            "#,
            input.user_id,
            input.friend_a_id,
            input.friend_b_id,
            input.a_to_b,
            input.b_to_a
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(relationship)
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateFriendRelationshipInput,
    ) -> Result<FriendRelationship, RepositoryError> {
        let relationship = sqlx::query_as!(
            FriendRelationship,
            r#"
            UPDATE friend_relationships
            SET
                a_to_b = COALESCE($2, a_to_b),
                b_to_a = COALESCE($3, b_to_a)
            WHERE id = $1
            RETURNING id, user_id, friend_a_id, friend_b_id, a_to_b, b_to_a, created_at, updated_at
            "#,
            id,
            input.a_to_b,
            input.b_to_a
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
            DELETE FROM friend_relationships
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
