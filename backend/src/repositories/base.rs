//! # Repository Base Types
//!
//! This module defines the core types used by all repositories:
//! - `RepositoryContext` - Holds the database connection pool
//! - `Repository` trait - Generic CRUD interface

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::error::RepositoryError;

/// Shared context for all repositories.
///
/// # Purpose
///
/// `RepositoryContext` holds the database connection pool and provides
/// helper methods for database operations. It's passed to each repository
/// constructor, allowing all repositories to share the same connection pool.
///
/// # Why Clone?
///
/// `PgPool` is internally an `Arc<PoolInner>`, so cloning is cheap (just
/// incrementing a reference count). This allows us to derive Clone on
/// RepositoryContext and share it across multiple repositories.
///
/// # Example
///
/// ```rust,ignore
/// let pool = PgPool::connect(&database_url).await?;
/// let ctx = RepositoryContext::new(pool);
///
/// let user_repo = UserRepository::new(ctx.clone());
/// let friend_repo = FriendRepository::new(ctx.clone());
/// ```
#[derive(Clone)]
pub struct RepositoryContext {
    /// The PostgreSQL connection pool.
    /// This is an Arc internally, so cloning is cheap.
    pub pool: PgPool,
}

impl RepositoryContext {
    /// Create a new RepositoryContext with the given connection pool.
    ///
    /// # Arguments
    ///
    /// * `pool` - A SQLx PostgreSQL connection pool
    ///
    /// # Returns
    ///
    /// A new RepositoryContext wrapping the pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Begin a new database transaction.
    ///
    /// # Purpose
    ///
    /// Transactions allow you to execute multiple database operations
    /// atomically - either all succeed or all are rolled back. This is
    /// essential for maintaining data consistency.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut tx = ctx.transaction().await?;
    ///
    /// // Execute multiple operations...
    /// sqlx::query!("INSERT INTO ...").execute(&mut *tx).await?;
    /// sqlx::query!("UPDATE ...").execute(&mut *tx).await?;
    ///
    /// // Commit if all succeeded
    /// tx.commit().await?;
    /// ```
    ///
    /// # Note on Lifetime
    ///
    /// The returned Transaction has a lifetime tied to the pool, not the
    /// RepositoryContext. This means the transaction can outlive the
    /// context if needed.
    ///
    /// # Errors
    ///
    /// Returns a RepositoryError if the transaction cannot be started
    /// (e.g., pool exhausted, database connection lost).
    pub async fn transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, RepositoryError> {
        self.pool.begin().await.map_err(RepositoryError::from_sqlx)
    }
}

/// Generic repository trait for CRUD operations.
///
/// # Type Parameters
///
/// Each implementation defines three associated types:
/// - `Entity` - The model type returned by queries (e.g., `User`)
/// - `CreateInput` - The input struct for creating new records
/// - `UpdateInput` - The input struct for updating existing records
///
/// # Why Associated Types?
///
/// Associated types (vs generics like `Repository<E, C, U>`) make the API
/// cleaner because each repository has exactly one Entity, CreateInput,
/// and UpdateInput. The types are determined by the implementation, not
/// the caller.
///
/// # Required Methods
///
/// All repositories must implement these four methods:
/// - `find_by_id` - Get a single record by primary key
/// - `create` - Insert a new record
/// - `update` - Modify an existing record
/// - `delete` - Remove a record
///
/// # Custom Methods
///
/// Repositories can (and should) add custom methods beyond this trait:
/// - `find_by_email` on UserRepository
/// - `list_by_user` on FriendRepository
/// - `list_by_friend` on FriendAttributeRepository
///
/// # async_trait
///
/// The `#[async_trait]` macro is required because Rust doesn't natively
/// support async functions in traits yet (as of 2024). This macro
/// transforms the async methods into a form the compiler accepts.
#[async_trait]
pub trait Repository: Send + Sync {
    /// The entity model returned by this repository
    type Entity;

    /// Input type for creating new records
    type CreateInput;

    /// Input type for updating existing records
    type UpdateInput;

    /// Find a record by its primary key.
    ///
    /// # Arguments
    ///
    /// * `id` - The UUID primary key to search for
    ///
    /// # Returns
    ///
    /// - `Ok(Some(entity))` if found
    /// - `Ok(None)` if no record exists with that ID
    /// - `Err(RepositoryError)` on database error
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Self::Entity>, RepositoryError>;

    /// Create a new record in the database.
    ///
    /// # Arguments
    ///
    /// * `input` - The data for the new record
    ///
    /// # Returns
    ///
    /// - `Ok(entity)` with the created record (including generated ID)
    /// - `Err(Duplicate)` if a unique constraint is violated
    /// - `Err(ForeignKeyViolation)` if a referenced record doesn't exist
    /// - `Err(Database)` on other database errors
    async fn create(&self, input: Self::CreateInput) -> Result<Self::Entity, RepositoryError>;

    /// Update an existing record.
    ///
    /// # Arguments
    ///
    /// * `id` - The UUID of the record to update
    /// * `input` - The new data for the record
    ///
    /// # Returns
    ///
    /// - `Ok(entity)` with the updated record
    /// - `Err(NotFound)` if no record exists with that ID
    /// - `Err(Duplicate)` if update violates a unique constraint
    /// - `Err(Database)` on other database errors
    async fn update(
        &self,
        id: Uuid,
        input: Self::UpdateInput,
    ) -> Result<Self::Entity, RepositoryError>;

    /// Delete a record from the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The UUID of the record to delete
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the record was deleted
    /// - `Ok(false)` if no record existed with that ID
    /// - `Err(Database)` on database error
    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError>;
}
