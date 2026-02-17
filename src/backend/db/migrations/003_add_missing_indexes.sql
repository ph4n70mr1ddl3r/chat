-- Add missing database indexes for performance
-- Created: 2025-01-26
-- Version: 3.0

-- Index for querying pending messages by recipient
CREATE INDEX IF NOT EXISTS idx_messages_recipient_status ON messages(recipient_id, status) WHERE status IN ('pending', 'failed');

-- Update schema version
INSERT OR REPLACE INTO schema_metadata (version, description) VALUES (3, 'Added index for pending message queries');
