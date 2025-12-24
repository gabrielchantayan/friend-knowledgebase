//! # User Repository
//!
//! Repository for user database operations.
//! Handles CRUD operations and custom queries for the `users` table.

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::User;

use super::base::{Repository, RepositoryContext};
use super::error::RepositoryError;

/// Input for creating a new user.
///
/// # Fields
/// - `first_name`: User's first name
/// - `last_name`: User's last name
/// - `email`: Unique email address (validated for uniqueness by database)
/// - `password_hash`: Pre-hashed password (use bcrypt before passing here)
///
/// # Note
/// The password should already be hashed before creating this struct.
/// Never store plain-text passwords!
pub struct CreateUserInput {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
}

/// Input for updating an existing user.
///
/// All fields are optional - only provided fields will be updated.
/// This follows the "partial update" pattern common in REST APIs.
///
/// # Example
///
/// ```rust,ignore
/// // Only update email, leave other fields unchanged
/// let input = UpdateUserInput {
///     first_name: None,
///     last_name: None,
///     email: Some("new@example.com".to_string()),
///     password_hash: None,
/// };
/// ```
pub struct UpdateUserInput {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

/// Repository for user database operations.
///
/// # Usage
///
/// ```rust,ignore
/// let ctx = RepositoryContext::new(pool);
/// let repo = UserRepository::new(ctx);
///
/// // Create a user
/// let user = repo.create(CreateUserInput { ... }).await?;
///
/// // Find by email
/// let user = repo.find_by_email("user@example.com").await?;
/// ```
pub struct UserRepository {
    /// Database context containing the connection pool
    ctx: RepositoryContext,
}

impl UserRepository {
    /// Create a new UserRepository with the given context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The repository context containing the database pool
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    /// Find a user by their email address.
    ///
    /// This is a custom finder beyond the standard Repository trait,
    /// since email is a common lookup field for authentication.
    ///
    /// # Arguments
    ///
    /// * `email` - The email address to search for
    ///
    /// # Returns
    ///
    /// - `Ok(Some(user))` if found
    /// - `Ok(None)` if no user has that email
    /// - `Err(RepositoryError)` on database error
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        // sqlx::query_as! maps the result directly to our User struct.
        // The query is validated at compile time against the database schema.
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, first_name, last_name, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(user)
    }
}

#[async_trait]
impl Repository for UserRepository {
    type Entity = User;
    type CreateInput = CreateUserInput;
    type UpdateInput = UpdateUserInput;

    /// Find a user by their UUID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, first_name, last_name, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(user)
    }

    /// Create a new user.
    ///
    /// # Errors
    ///
    /// Returns `Duplicate` if the email already exists (unique constraint).
    async fn create(&self, input: CreateUserInput) -> Result<User, RepositoryError> {
        // INSERT ... RETURNING gives us the created row back, including
        // the generated UUID and timestamps.
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (first_name, last_name, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, first_name, last_name, email, password_hash, created_at, updated_at
            "#,
            input.first_name,
            input.last_name,
            input.email,
            input.password_hash
        )
        .fetch_one(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?;

        Ok(user)
    }

    /// Update an existing user.
    ///
    /// Uses COALESCE to only update fields that are provided (not NULL).
    /// This is a common pattern for partial updates.
    async fn update(&self, id: Uuid, input: UpdateUserInput) -> Result<User, RepositoryError> {
        // COALESCE returns the first non-NULL argument.
        // So COALESCE($2, first_name) means: use $2 if provided, else keep current value.
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                email = COALESCE($4, email),
                password_hash = COALESCE($5, password_hash)
            WHERE id = $1
            RETURNING id, first_name, last_name, email, password_hash, created_at, updated_at
            "#,
            id,
            input.first_name,
            input.last_name,
            input.email,
            input.password_hash
        )
        .fetch_optional(&self.ctx.pool)
        .await
        .map_err(RepositoryError::from_sqlx)?
        .ok_or(RepositoryError::NotFound)?;

        Ok(user)
    }

    /// Delete a user by their UUID.
    ///
    /// Due to cascading deletes, this will also delete all of the user's
    /// friends, groups, and related data.
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        // result.rows_affected() tells us how many rows were deleted.
        // Should be 0 or 1 since id is a primary key.
        let result = sqlx::query!(
            r#"
            DELETE FROM users
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
