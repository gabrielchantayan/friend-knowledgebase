//! # Repository Error Types
//!
//! This module defines the error types for repository operations.
//! It maps low-level database errors to semantic domain errors.

use thiserror::Error;

/// Error type for repository operations.
///
/// # Variants
///
/// - `NotFound` - The requested record doesn't exist
/// - `Duplicate` - A unique constraint was violated (e.g., duplicate email)
/// - `ForeignKeyViolation` - Referenced record doesn't exist
/// - `Database` - Generic database error
/// - `Serialization` - JSON serialization/deserialization failed
///
/// # PostgreSQL Error Codes
///
/// This error type maps PostgreSQL error codes to semantic variants:
/// - `23505` → `Duplicate` (unique_violation)
/// - `23503` → `ForeignKeyViolation` (foreign_key_violation)
///
/// # Example
///
/// ```rust,ignore
/// match repository.create(input).await {
///     Ok(user) => println!("Created user: {}", user.email),
///     Err(RepositoryError::Duplicate(msg)) => println!("Email already exists: {}", msg),
///     Err(e) => println!("Database error: {}", e),
/// }
/// ```
#[derive(Error, Debug)]
pub enum RepositoryError {
    /// The requested record was not found in the database
    #[error("Record not found")]
    NotFound,

    /// A unique constraint was violated (e.g., duplicate email)
    /// The string contains details about which constraint was violated
    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    /// A foreign key constraint was violated (referenced record doesn't exist)
    /// The string contains details about which foreign key failed
    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),

    /// A generic database error that doesn't fit other categories
    /// Wraps the underlying SQLx error for debugging
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// JSON serialization or deserialization failed
    /// This can happen when working with JSONB columns
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl RepositoryError {
    /// Convert a SQLx error into a RepositoryError.
    ///
    /// This function inspects the PostgreSQL error code to determine
    /// the appropriate RepositoryError variant. PostgreSQL uses
    /// 5-character SQLSTATE codes defined in the SQL standard.
    ///
    /// # PostgreSQL Error Codes
    ///
    /// - `23505` - unique_violation: Attempt to insert/update a duplicate value
    /// - `23503` - foreign_key_violation: Referenced key doesn't exist
    ///
    /// See: https://www.postgresql.org/docs/current/errcodes-appendix.html
    ///
    /// # Arguments
    ///
    /// * `err` - The SQLx error to convert
    ///
    /// # Returns
    ///
    /// A RepositoryError variant based on the error code, or Database for unknown errors
    pub fn from_sqlx(err: sqlx::Error) -> Self {
        match &err {
            // RowNotFound is a specific SQLx error when a query expecting 1 row gets 0
            sqlx::Error::RowNotFound => RepositoryError::NotFound,

            // Database errors contain the actual PostgreSQL error with SQLSTATE code
            sqlx::Error::Database(db_err) => {
                // db_err.code() returns the 5-character SQLSTATE code
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        // 23505: unique_violation - duplicate key value
                        "23505" => RepositoryError::Duplicate(db_err.message().to_string()),

                        // 23503: foreign_key_violation - referenced key doesn't exist
                        "23503" => {
                            RepositoryError::ForeignKeyViolation(db_err.message().to_string())
                        }

                        // Unknown error code - wrap as generic Database error
                        _ => RepositoryError::Database(err),
                    }
                } else {
                    // No error code available - wrap as generic Database error
                    RepositoryError::Database(err)
                }
            }

            // All other SQLx errors become generic Database errors
            _ => RepositoryError::Database(err),
        }
    }
}
