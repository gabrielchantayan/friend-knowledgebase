//! # Friend Repository
//!
//! Repository for friend database operations.
//! This is the core entity of FKB - handles CRUD and group membership.

use async_trait::async_trait;
use time::Date;
use uuid::Uuid;

use crate::models::{Friend, Group};

use super::base::{Repository, RepositoryContext};
use super::error::RepositoryError;

/// Input for creating a new friend.
pub struct CreateFriendInput {
    /// The user who owns this friend record
    pub user_id: Uuid,
    /// Friend's first name (required)
    pub first_name: String,
    /// Friend's last name (optional)
    pub last_name: Option<String>,
    /// Friend's date of birth for birthday reminders
    pub date_of_birth: Option<Date>,
    /// Things the friend likes
    pub likes: Option<String>,
    /// Things the friend dislikes
    pub dislikes: Option<String>,
    /// General notes about the friend
    pub notes: Option<String>,
}

/// Input for updating an existing friend.
/// All fields optional - only provided fields are updated.
pub struct UpdateFriendInput {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<Date>,
    pub likes: Option<String>,
    pub dislikes: Option<String>,
    pub notes: Option<String>,
}

/// Repository for friend database operations.
///
/// # Group Membership
///
/// This repository also handles the friend-group relationship (many-to-many).
/// Methods like `add_to_group`, `remove_from_group`, and `list_groups` manage
/// the `friend_groups` join table.
pub struct FriendRepository {
    ctx: RepositoryContext,
}

impl FriendRepository {
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    /// List all friends for a given user.
    ///
    /// This is the primary query for the friends list view.
    /// Results are ordered by first name for consistent display.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The UUID of the user whose friends to list
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Friend>, RepositoryError> {
        let friends = sqlx::query_as!(
            Friend,
            r#"
            SELECT id, user_id, first_name, last_name, date_of_birth,
                   likes, dislikes, notes, created_at, updated_at
            FROM friends
            WHERE user_id = $1
            ORDER BY first_name ASC
            "#,
            user_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(friends)
    }

    /// Add a friend to a group.
    ///
    /// # Arguments
    ///
    /// * `friend_id` - The friend to add
    /// * `group_id` - The group to add them to
    ///
    /// # Note
    ///
    /// Uses ON CONFLICT DO NOTHING to make this idempotent - calling
    /// multiple times with the same IDs is safe.
    pub async fn add_to_group(
        &self,
        friend_id: Uuid,
        group_id: Uuid,
    ) -> Result<(), RepositoryError> {
        // ON CONFLICT DO NOTHING makes this idempotent.
        // If the friend is already in the group, this does nothing.
        sqlx::query!(
            r#"
            INSERT INTO friend_groups (friend_id, group_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            friend_id,
            group_id
        )
        .execute(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(())
    }

    /// Remove a friend from a group.
    ///
    /// # Returns
    ///
    /// `true` if the friend was in the group and removed,
    /// `false` if they weren't in the group.
    pub async fn remove_from_group(
        &self,
        friend_id: Uuid,
        group_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM friend_groups
            WHERE friend_id = $1 AND group_id = $2
            "#,
            friend_id,
            group_id
        )
        .execute(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(result.rows_affected() > 0)
    }

    /// List all groups a friend belongs to.
    ///
    /// # Returns
    ///
    /// A vector of Group entities the friend is a member of.
    pub async fn list_groups(&self, friend_id: Uuid) -> Result<Vec<Group>, RepositoryError> {
        // JOIN through friend_groups to get the actual Group entities
        let groups = sqlx::query_as!(
            Group,
            r#"
            SELECT g.id, g.user_id, g.name, g.description, g.created_at, g.updated_at
            FROM groups g
            INNER JOIN friend_groups fg ON fg.group_id = g.id
            WHERE fg.friend_id = $1
            ORDER BY g.name ASC
            "#,
            friend_id
        )
        .fetch_all(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(groups)
    }
}

#[async_trait]
impl Repository for FriendRepository {
    type Entity = Friend;
    type CreateInput = CreateFriendInput;
    type UpdateInput = UpdateFriendInput;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Friend>, RepositoryError> {
        let friend = sqlx::query_as!(
            Friend,
            r#"
            SELECT id, user_id, first_name, last_name, date_of_birth,
                   likes, dislikes, notes, created_at, updated_at
            FROM friends
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(friend)
    }

    async fn create(&self, input: CreateFriendInput) -> Result<Friend, RepositoryError> {
        let friend = sqlx::query_as!(
            Friend,
            r#"
            INSERT INTO friends (user_id, first_name, last_name, date_of_birth, likes, dislikes, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, first_name, last_name, date_of_birth,
                      likes, dislikes, notes, created_at, updated_at
            "#,
            input.user_id,
            input.first_name,
            input.last_name,
            input.date_of_birth,
            input.likes,
            input.dislikes,
            input.notes
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(friend)
    }

    async fn update(&self, id: Uuid, input: UpdateFriendInput) -> Result<Friend, RepositoryError> {
        let friend = sqlx::query_as!(
            Friend,
            r#"
            UPDATE friends
            SET
                first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                date_of_birth = COALESCE($4, date_of_birth),
                likes = COALESCE($5, likes),
                dislikes = COALESCE($6, dislikes),
                notes = COALESCE($7, notes)
            WHERE id = $1
            RETURNING id, user_id, first_name, last_name, date_of_birth,
                      likes, dislikes, notes, created_at, updated_at
            "#,
            id,
            input.first_name,
            input.last_name,
            input.date_of_birth,
            input.likes,
            input.dislikes,
            input.notes
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(friend)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM friends
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
