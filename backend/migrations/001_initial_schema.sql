-- Friend Knowledgebase Initial Schema
-- Migration: 001_initial_schema.sql

-- =============================================================================
-- TRIGGER FUNCTION
-- =============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- TABLES
-- =============================================================================

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- Friends table
CREATE TABLE friends (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR,
    date_of_birth DATE,
    likes TEXT,
    dislikes TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- Groups table
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- Friend-Group join table (many-to-many)
CREATE TABLE friend_groups (
    friend_id UUID NOT NULL REFERENCES friends(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    PRIMARY KEY (friend_id, group_id)
);

-- Friend attributes (flexible key-value store)
CREATE TABLE friend_attributes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    friend_id UUID NOT NULL REFERENCES friends(id) ON DELETE CASCADE,
    key VARCHAR NOT NULL,
    value TEXT NOT NULL,
    value_type VARCHAR NOT NULL DEFAULT 'text',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    UNIQUE (friend_id, key)
);

-- Friend-to-friend relationships
CREATE TABLE friend_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_a_id UUID NOT NULL REFERENCES friends(id) ON DELETE CASCADE,
    friend_b_id UUID NOT NULL REFERENCES friends(id) ON DELETE CASCADE,
    a_to_b TEXT NOT NULL,
    b_to_a TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- User-to-friend relationships (how user knows each friend)
CREATE TABLE user_friend_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    friend_id UUID NOT NULL REFERENCES friends(id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- Friends indexes
CREATE INDEX idx_friends_user_id ON friends(user_id);

-- Groups indexes
CREATE INDEX idx_groups_user_id ON groups(user_id);

-- Friend-groups indexes
CREATE INDEX idx_friend_groups_group_id ON friend_groups(group_id);

-- Friend attributes indexes
CREATE INDEX idx_friend_attributes_friend_id ON friend_attributes(friend_id);

-- Friend relationships indexes
CREATE INDEX idx_friend_relationships_user_id ON friend_relationships(user_id);
CREATE INDEX idx_friend_relationships_friend_a ON friend_relationships(friend_a_id);
CREATE INDEX idx_friend_relationships_friend_b ON friend_relationships(friend_b_id);

-- User-friend relationships indexes
CREATE INDEX idx_user_friend_relationships_friend_id ON user_friend_relationships(friend_id);

-- =============================================================================
-- TRIGGERS
-- =============================================================================

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_friends_updated_at
    BEFORE UPDATE ON friends
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_groups_updated_at
    BEFORE UPDATE ON groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_friend_attributes_updated_at
    BEFORE UPDATE ON friend_attributes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_friend_relationships_updated_at
    BEFORE UPDATE ON friend_relationships
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_friend_relationships_updated_at
    BEFORE UPDATE ON user_friend_relationships
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
