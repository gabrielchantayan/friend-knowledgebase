## Project Structure

- models
    - models/repositories — database access
    - models/types — types and schema
- services — business logic
- utils — utility functions
- controllers — API endpoints

## Database

- Use UUID as the primary key. Prefer UUIDv7
- Use TIMESTAMPTZ for all timestamps
- `created_at` defaults to `NOW()`
- `updated_at` uses the trigger `update_updated_at_column`
