## Project Structure

- models — data models
- repositories — data access
- services — business logic
- utils — utility functions
- controllers — API endpoints

## Code Style

Comment everything. I am a novice Rust developer, coming from an experienced TypeScript background.
I am aware of the basics of Rust, but I am still learning. This project is a part of my learning process.
I intend for it to be production-ready, but still a place for me to learn. Explain in the comments
what is going on, even if it is simple. This will help aide in my Rust learning journey.

## Database

- Use UUID as the primary key. Prefer UUIDv7
- Use TIMESTAMPTZ for all timestamps
- `created_at` defaults to `NOW()`
- `updated_at` uses the trigger `update_updated_at_column`

## Models Pattern

Models are simple structs representing database entities. Located in `src/models/`.

- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` on all models
- Use `time::OffsetDateTime` for TIMESTAMPTZ columns
- Use `time::Date` for DATE columns
- Use `Option<T>` for nullable columns
- Use `#[serde(skip_serializing)]` on sensitive fields like `password_hash`
- One file per entity (e.g., `user.rs`, `friend.rs`)
- Re-export all models from `mod.rs`

Example:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
}
```

## Repository Pattern

Repositories handle all database operations. Located in `src/repositories/`.

### Core Components

- **`RepositoryContext`** - Holds `PgPool`, provides `transaction()` helper
- **`Repository` trait** - Generic CRUD interface with associated types:
  - `Entity` - The model type returned
  - `CreateInput` - Input struct for creating records
  - `UpdateInput` - Input struct for updating records
- **`RepositoryError`** - Error enum that maps PostgreSQL error codes:
  - `23505` → `Duplicate` (unique constraint violation)
  - `23503` → `ForeignKeyViolation`
  - `NotFound`, `Database`, `Serialization`

### Standard Methods

Trait methods:
- `find_by_id(id: Uuid)` - Get single record
- `create(input: CreateInput)` - Insert new record
- `update(id: Uuid, input: UpdateInput)` - Update existing record
- `delete(id: Uuid)` - Remove record

Custom finders (beyond trait):
- `find_by_email`, `list_by_user`, `list_by_friend`, etc.

### Implementation Notes

- Use `async_trait` crate for async trait methods
- Use `sqlx::query!` macro for compile-time SQL validation
- Each repository gets its own file (e.g., `user_repository.rs`)
- Constructor takes `RepositoryContext`

Example:
```rust
pub struct UserRepository {
    ctx: RepositoryContext,
}

impl UserRepository {
    pub fn new(ctx: RepositoryContext) -> Self {
        Self { ctx }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        // sqlx::query! implementation
    }
}
```
