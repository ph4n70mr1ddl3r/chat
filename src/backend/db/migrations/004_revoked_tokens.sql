-- Add revoked_tokens table for persistent token revocation
-- Created: 2026-03-14
-- Version: 4.0

CREATE TABLE IF NOT EXISTS revoked_tokens (
  token_hash TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  revoked_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_user_id ON revoked_tokens(user_id);

INSERT OR IGNORE INTO schema_metadata (version, description) VALUES (4, 'Add revoked_tokens table for persistent token revocation');
