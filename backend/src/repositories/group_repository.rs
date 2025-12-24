//! # Group Repository
//!
//! Repository for group database operations.
//! Groups help organize friends into categories.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{Friend, Group};

use super::base::{Repository, RepositoryContext};
use super::error::RepositoryError;

/// Input for creating a new group.
pub struct CreateGroupInput {
    /// The user who owns this group
    pub user_id: Uuid,
    /// Display name for the group (e.g., "Work", "Family")
    pub name: String,
    /// Optional description of the group
    pub description: Option<String>,
}

/// Input for updating an existing group.
pub struct UpdateGroupInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Repository for group database operations.
///
/// # Friend Membership
///
/// This repository includes `list_friends` to get all friends in a group.
/// Adding/removing friends is handled by `FriendRepository` since the
/// operation is typically done from the friend's perspective.
pub struct GroupRepository {
    ctx: RepositoryContext,
}

impl GroupRepository {
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    /// List all groups for a given user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The UUID of the user whose groups to list
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Group>, RepositoryError> {
        let groups = sqlx::query_as!(
            Group,
            r#"
            SELECT id, user_id, name, description, created_at, updated_at
            FROM groups
            WHERE user_id = $1
            ORDER BY name ASC
            "#,
            user_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(groups)
    }

    /// List all friends in a group.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The UUID of the group
    ///
    /// # Returns
    ///
    /// A vector of Friend entities that belong to this group.
    pub async fn list_friends(&self, group_id: Uuid) -> Result<Vec<Friend>, RepositoryError> {
        let friends = sqlx::query_as!(
            Friend,
            r#"
            SELECT f.id, f.user_id, f.first_name, f.last_name, f.date_of_birth,
                   f.likes, f.dislikes, f.notes, f.created_at, f.updated_at
            FROM friends f
            INNER JOIN friend_groups fg ON fg.friend_id = f.id
            WHERE fg.group_id = $1
            ORDER BY f.first_name ASC
            "#,
            group_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(friends)
    }
}

#[async_trait]
impl Repository for GroupRepository {
    type Entity = Group;
    type CreateInput = CreateGroupInput;
    type UpdateInput = UpdateGroupInput;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Group>, RepositoryError> {
        let group = sqlx::query_as!(
            Group,
            r#"
            SELECT id, user_id, name, description, created_at, updated_at
            FROM groups
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(group)
    }

    async fn create(&self, input: CreateGroupInput) -> Result<Group, RepositoryError> {
        let group = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (user_id, name, description)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, name, description, created_at, updated_at
            "#,
            input.user_id,
            input.name,
            input.description
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(group)
    }

    async fn update(&self, id: Uuid, input: UpdateGroupInput) -> Result<Group, RepositoryError> {
        let group = sqlx::query_as!(
            Group,
            r#"
            UPDATE groups
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description)
            WHERE id = $1
            RETURNING id, user_id, name, description, created_at, updated_at
            "#,
            id,
            input.name,
            input.description
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(group)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM groups
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
