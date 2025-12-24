//! # Friend Attribute Repository
//!
//! Repository for friend attribute database operations.
//! Attributes are key-value pairs for storing custom friend data.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::FriendAttribute;

use super::base::{Repository, RepositoryContext};
use super::error::RepositoryError;

/// Input for creating a new friend attribute.
pub struct CreateFriendAttributeInput {
    /// The friend this attribute belongs to
    pub friend_id: Uuid,
    /// Attribute key (unique per friend)
    pub key: String,
    /// Attribute value as text
    pub value: String,
    /// Type hint for the value (default: "text")
    pub value_type: Option<String>,
}

/// Input for updating an existing friend attribute.
pub struct UpdateFriendAttributeInput {
    pub value: Option<String>,
    pub value_type: Option<String>,
}

/// Repository for friend attribute database operations.
pub struct FriendAttributeRepository {
    ctx: RepositoryContext,
}

impl FriendAttributeRepository {
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    /// List all attributes for a friend.
    ///
    /// # Arguments
    ///
    /// * `friend_id` - The UUID of the friend
    pub async fn list_by_friend(
        &self,
        friend_id: Uuid,
    ) -> Result<Vec<FriendAttribute>, RepositoryError> {
        let attributes = sqlx::query_as!(
            FriendAttribute,
            r#"
            SELECT id, friend_id, key, value, value_type, created_at, updated_at
            FROM friend_attributes
            WHERE friend_id = $1
            ORDER BY key ASC
            "#,
            friend_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(attributes)
    }

    /// Find an attribute by friend and key.
    ///
    /// This is useful for checking if an attribute exists before creating it,
    /// or for getting a specific attribute value.
    ///
    /// # Arguments
    ///
    /// * `friend_id` - The UUID of the friend
    /// * `key` - The attribute key to find
    pub async fn find_by_friend_and_key(
        &self,
        friend_id: Uuid,
        key: &str,
    ) -> Result<Option<FriendAttribute>, RepositoryError> {
        let attribute = sqlx::query_as!(
            FriendAttribute,
            r#"
            SELECT id, friend_id, key, value, value_type, created_at, updated_at
            FROM friend_attributes
            WHERE friend_id = $1 AND key = $2
            "#,
            friend_id,
            key
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(attribute)
    }

    /// Create or update an attribute (upsert).
    ///
    /// If an attribute with the same friend_id and key exists, it's updated.
    /// Otherwise, a new attribute is created.
    ///
    /// # Arguments
    ///
    /// * `input` - The attribute data
    pub async fn upsert(
        &self,
        input: CreateFriendAttributeInput,
    ) -> Result<FriendAttribute, RepositoryError> {
        // ON CONFLICT ... DO UPDATE is PostgreSQL's upsert syntax.
        // It inserts if no conflict, or updates if there's a duplicate key.
        let value_type = input.value_type.unwrap_or_else(|| "text".to_string());

        let attribute = sqlx::query_as!(
            FriendAttribute,
            r#"
            INSERT INTO friend_attributes (friend_id, key, value, value_type)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (friend_id, key) DO UPDATE
            SET value = EXCLUDED.value, value_type = EXCLUDED.value_type
            RETURNING id, friend_id, key, value, value_type, created_at, updated_at
            "#,
            input.friend_id,
            input.key,
            input.value,
            value_type
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(attribute)
    }
}

#[async_trait]
impl Repository for FriendAttributeRepository {
    type Entity = FriendAttribute;
    type CreateInput = CreateFriendAttributeInput;
    type UpdateInput = UpdateFriendAttributeInput;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FriendAttribute>, RepositoryError> {
        let attribute = sqlx::query_as!(
            FriendAttribute,
            r#"
            SELECT id, friend_id, key, value, value_type, created_at, updated_at
            FROM friend_attributes
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(attribute)
    }

    async fn create(
        &self,
        input: CreateFriendAttributeInput,
    ) -> Result<FriendAttribute, RepositoryError> {
        let value_type = input.value_type.unwrap_or_else(|| "text".to_string());

        let attribute = sqlx::query_as!(
            FriendAttribute,
            r#"
            INSERT INTO friend_attributes (friend_id, key, value, value_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, friend_id, key, value, value_type, created_at, updated_at
            "#,
            input.friend_id,
            input.key,
            input.value,
            value_type
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(attribute)
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateFriendAttributeInput,
    ) -> Result<FriendAttribute, RepositoryError> {
        let attribute = sqlx::query_as!(
            FriendAttribute,
            r#"
            UPDATE friend_attributes
            SET
                value = COALESCE($2, value),
                value_type = COALESCE($3, value_type)
            WHERE id = $1
            RETURNING id, friend_id, key, value, value_type, created_at, updated_at
            "#,
            id,
            input.value,
            input.value_type
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(attribute)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM friend_attributes
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
