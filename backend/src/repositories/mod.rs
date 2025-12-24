//! # Repositories Module
//!
//! This module contains the data access layer for the Friend Knowledgebase.
//! Repositories handle all database operations using SQLx with PostgreSQL.
//!
//! ## Architecture
//!
//! ```text
//! Controllers → Services → Repositories → Database
//! ```
//!
//! Repositories are the only layer that directly interacts with the database.
//! They return domain models and handle SQL queries.
//!
//! ## Core Components
//!
//! - `RepositoryContext` - Holds the database connection pool
//! - `Repository` trait - Generic CRUD interface
//! - `RepositoryError` - Error types for database operations
//!
//! ## Pattern
//!
//! Each repository:
//! 1. Takes `RepositoryContext` in constructor
//! 2. Implements the `Repository` trait for standard CRUD
//! 3. Adds custom finder methods as needed (e.g., `find_by_email`)
//! 4. Uses `sqlx::query!` macro for compile-time SQL validation

// Core infrastructure
pub mod base;
pub mod error;

// Entity repositories
pub mod user_repository;
pub mod friend_repository;
pub mod group_repository;
pub mod friend_attribute_repository;
pub mod friend_relationship_repository;
pub mod user_friend_relationship_repository;

// Re-export core types for convenient access
pub use base::{Repository, RepositoryContext};
pub use error::RepositoryError;

// Re-export repositories
pub use user_repository::{UserRepository, CreateUserInput, UpdateUserInput};
pub use friend_repository::{FriendRepository, CreateFriendInput, UpdateFriendInput};
pub use group_repository::{GroupRepository, CreateGroupInput, UpdateGroupInput};
pub use friend_attribute_repository::{FriendAttributeRepository, CreateFriendAttributeInput, UpdateFriendAttributeInput};
pub use friend_relationship_repository::{FriendRelationshipRepository, CreateFriendRelationshipInput, UpdateFriendRelationshipInput};
pub use user_friend_relationship_repository::{UserFriendRelationshipRepository, CreateUserFriendRelationshipInput, UpdateUserFriendRelationshipInput};
