# Database Schema Design

## Overview

PostgreSQL database schema for FKB with multi-user support and data isolation.

## Requirements

- UUIDs for all primary keys
- Multi-user support with siloed data
- TIMESTAMPTZ for all timestamps
- `created_at` defaults to `now()`
- `updated_at` auto-managed via trigger

## Tables

| Table | Purpose |
|-------|---------|
| users | Application users |
| friends | Friend records (core entity) |
| groups | User-defined groups for organizing friends |
| friend_groups | Many-to-many join table |
| friend_attributes | Flexible key-value store for custom attributes |
| friend_relationships | Friend-to-friend relationships (symmetric/asymmetric) |
| user_friend_relationships | How user knows each friend |

## Data Isolation

All user-owned data filtered by `user_id`:
- `friends`, `groups`, `friend_relationships` - direct column
- `friend_groups`, `friend_attributes`, `user_friend_relationships` - join through friends

## Relationship Logic

**Friend-to-friend:**
- `a_to_b`: Required (e.g., "hates", "sibling of")
- `b_to_a`: Optional. NULL = symmetric, set = asymmetric

**User-to-friend:**
- `relationship_type`: How user knows them (e.g., "coworker")

## Implementation

Migration file: `backend/migrations/001_initial_schema.sql`
