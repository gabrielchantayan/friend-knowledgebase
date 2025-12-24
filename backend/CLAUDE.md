## Project Structure

- models
    - models/repositories — database access
    - models/types — types and schema
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
