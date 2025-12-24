//! # Models Module
//!
//! This module contains all database entity models for the Friend Knowledgebase.
//! Each model is a Rust struct that maps directly to a database table.
//!
//! ## Pattern
//! - One file per entity
//! - All models derive Debug, Clone, Serialize, Deserialize
//! - Use Option<T> for nullable columns
//! - Use time::OffsetDateTime for timestamps, time::Date for dates

// Model definitions
pub mod user;
pub mod friend;
pub mod group;
pub mod friend_attribute;
pub mod friend_relationship;
pub mod user_friend_relationship;

// Re-export all models for convenient access
// e.g., `use crate::models::User;` instead of `use crate::models::user::User;`
pub use user::User;
pub use friend::Friend;
pub use group::Group;
pub use friend_attribute::FriendAttribute;
pub use friend_relationship::FriendRelationship;
pub use user_friend_relationship::UserFriendRelationship;
