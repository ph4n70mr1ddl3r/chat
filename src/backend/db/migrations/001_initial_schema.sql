-- Private Chat Application Initial Schema
-- Created: 2025-12-15
-- Version: 1.0

-- Users table
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  password_salt TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (cast(strftime('%s', 'now') as integer) * 1000),
  updated_at INTEGER NOT NULL DEFAULT (cast(strftime('%s', 'now') as integer) * 1000),
  deleted_at INTEGER,
  is_online BOOLEAN NOT NULL DEFAULT FALSE,
  last_seen_at INTEGER,
  CHECK (length(username) >= 1 AND length(username) <= 50),
  CHECK (length(password_hash) > 0)
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at);

-- Conversations table (one-to-one chats)
CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  user1_id TEXT NOT NULL,
  user2_id TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (cast(strftime('%s', 'now') as integer) * 1000),
  updated_at INTEGER NOT NULL DEFAULT (cast(strftime('%s', 'now') as integer) * 1000),
  last_message_at INTEGER,
  message_count INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (user1_id) REFERENCES users(id),
  FOREIGN KEY (user2_id) REFERENCES users(id),
  UNIQUE (user1_id, user2_id),
  CHECK (user1_id < user2_id),
  CHECK (user1_id != user2_id)
);

CREATE INDEX IF NOT EXISTS idx_conversations_user1_id ON conversations(user1_id);
CREATE INDEX IF NOT EXISTS idx_conversations_user2_id ON conversations(user2_id);
CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at DESC);

-- Messages table
CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  recipient_id TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (cast(strftime('%s', 'now') as integer) * 1000),
  delivered_at INTEGER,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'delivered', 'failed')),
  is_anonymized BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id),
  FOREIGN KEY (sender_id) REFERENCES users(id),
  FOREIGN KEY (recipient_id) REFERENCES users(id),
  CHECK (length(content) >= 1 AND length(content) <= 5000)
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status) WHERE status IN ('pending', 'failed');
CREATE INDEX IF NOT EXISTS idx_messages_delivered_at ON messages(delivered_at) WHERE delivered_at IS NULL;

-- Metadata table for schema versioning
CREATE TABLE IF NOT EXISTS schema_metadata (
  version INTEGER PRIMARY KEY,
  description TEXT NOT NULL,
  applied_at INTEGER NOT NULL DEFAULT (cast(strftime('%s', 'now') as integer) * 1000)
);

INSERT OR IGNORE INTO schema_metadata (version, description) VALUES (1, 'Initial schema with users, conversations, messages tables');
