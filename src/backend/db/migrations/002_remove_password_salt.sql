-- Migration: Remove deprecated password_salt column
-- Date: 2025-01-29
-- Description: Remove the password_salt column as it is deprecated.
--              bcrypt includes salt internally in password_hash.

-- Remove password_salt column from users table
-- Note: This migration is designed to be idempotent
-- For SQLite, we need to recreate the table without the column
CREATE TABLE IF NOT EXISTS users_new (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  is_online BOOLEAN NOT NULL DEFAULT FALSE,
  last_seen_at INTEGER,
  CHECK (length(username) >= 1 AND length(username) <= 50),
  CHECK (length(password_hash) > 0)
);

-- Copy data from old table to new table (excluding password_salt)
INSERT INTO users_new (id, username, password_hash, created_at, updated_at, deleted_at, is_online, last_seen_at)
SELECT id, username, password_hash, created_at, updated_at, deleted_at, is_online, last_seen_at
FROM users;

-- Drop old table
DROP TABLE IF EXISTS users;

-- Rename new table to original name
ALTER TABLE users_new RENAME TO users;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at);
