# PostgreSQL Migration Strategy

**Version**: 1.0.0  
**Last Updated**: 2025-12-16  
**Status**: Planning Document  

---

## Table of Contents

1. [Overview](#overview)
2. [Migration Rationale](#migration-rationale)
3. [Current State (SQLite)](#current-state-sqlite)
4. [Target State (PostgreSQL)](#target-state-postgresql)
5. [Migration Strategy](#migration-strategy)
6. [Schema Changes](#schema-changes)
7. [Data Migration Process](#data-migration-process)
8. [Testing & Validation](#testing--validation)
9. [Rollback Plan](#rollback-plan)
10. [Performance Considerations](#performance-considerations)
11. [Timeline & Resources](#timeline--resources)

---

## Overview

This document outlines the strategy for migrating the Private Chat Application from SQLite (MVP) to PostgreSQL (production-grade) for improved scalability, concurrency, and performance.

### Current State

- **Database**: SQLite 3.35+ (single-file, WAL mode)
- **Deployment**: Single Linux server (MVP)
- **Capacity**: 100-1,000 concurrent users
- **Performance**: 100 messages/sec peak throughput

### Target State

- **Database**: PostgreSQL 14+ with async replication
- **Deployment**: Multi-node cluster with load balancing
- **Capacity**: 10,000+ concurrent users
- **Performance**: 1,000+ messages/sec peak throughput

---

## Migration Rationale

### Why Migrate?

1. **Scalability Limits**
   - SQLite: Single-writer model limits write throughput
   - PostgreSQL: Multi-user MVCC allows concurrent writes
   - Target: 10x increase in concurrent users (1,000 → 10,000)

2. **Concurrency**
   - SQLite: Write locks block reads (even in WAL mode)
   - PostgreSQL: MVCC allows reads during writes
   - Benefit: Lower latency for message retrieval during peak traffic

3. **Replication & High Availability**
   - SQLite: No built-in replication
   - PostgreSQL: Streaming replication, standby servers, automatic failover
   - Benefit: 99.9% uptime target (vs. 95% with single SQLite server)

4. **Advanced Features**
   - PostgreSQL: Full-text search (tsvector), JSON columns, partitioning, connection pooling
   - Benefit: Better search performance, horizontal scaling via sharding

5. **Operational Maturity**
   - PostgreSQL: Industry-standard with extensive tooling (pgAdmin, pg_dump, WAL archiving)
   - Benefit: Better monitoring, backup strategies, disaster recovery

### Why Not Migrate Yet?

- **MVP Simplicity**: SQLite is sufficient for 100-1,000 users
- **Operational Overhead**: PostgreSQL requires dedicated database server, backups, monitoring
- **Cost**: PostgreSQL requires more infrastructure (separate DB server, replication, load balancer)

**Decision**: Migrate when user count exceeds 1,000 or write throughput approaches 100 msgs/sec.

---

## Current State (SQLite)

### Schema Overview

**Tables**:
1. `users` (id, username, password_hash, password_salt, created_at, updated_at, deleted_at, is_online, last_seen_at)
2. `conversations` (id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count)
3. `messages` (id, conversation_id, sender_id, recipient_id, content, created_at, delivered_at, status, is_anonymized)

**Indexes**:
- `idx_users_username` (unique)
- `idx_users_deleted_at`
- `idx_conversations_user1_id`, `idx_conversations_user2_id`
- `idx_conversations_updated_at`
- `idx_messages_conversation_id`
- `idx_messages_status`
- `idx_messages_delivered_at`

**Constraints**:
- Foreign keys: `conversations.user1_id`, `conversations.user2_id`, `messages.sender_id`, `messages.recipient_id`
- Unique: `users.username`, `conversations(user1_id, user2_id)`
- Check: `conversations.user1_id < user2_id` (prevent duplicate conversations)

### Current Limitations

1. **Write Throughput**: ~100 messages/sec (single writer)
2. **Read Latency**: Increases under write load (WAL checkpoint overhead)
3. **Database Size**: 10 GB+ becomes slow for queries (full table scans)
4. **Backup Downtime**: File-based backups require service interruption

---

## Target State (PostgreSQL)

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│   (Backend Servers: Load Balanced, Stateless)           │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────┐
│                Connection Pool (PgBouncer)               │
│              (Transaction Pooling, 100-200 conns)       │
└─────────────────┬───────────────────────────────────────┘
                  │
          ┌───────┴──────┐
          │              │
┌─────────▼──────┐ ┌────▼──────────┐
│  PostgreSQL    │ │  PostgreSQL   │
│   Primary      │ │  Standby      │
│ (Read + Write) │ │  (Read Only)  │
└────────┬───────┘ └───────────────┘
         │
         │ WAL Streaming Replication
         └─────────────────────────────────────┐
                                               │
                                    ┌──────────▼──────┐
                                    │  PostgreSQL     │
                                    │  Standby (Hot)  │
                                    │  (Failover)     │
                                    └─────────────────┘
```

### PostgreSQL Configuration

**Hardware**:
- Primary: 8 CPU cores, 32 GB RAM, 500 GB NVMe SSD
- Standby: Same as primary (for failover)

**PostgreSQL Settings** (`postgresql.conf`):
```ini
max_connections = 200
shared_buffers = 8GB
effective_cache_size = 24GB
maintenance_work_mem = 2GB
work_mem = 64MB

# WAL settings
wal_level = replica
max_wal_senders = 3
wal_keep_size = 1GB

# Checkpointing
checkpoint_timeout = 15min
max_wal_size = 4GB
min_wal_size = 1GB

# Performance
random_page_cost = 1.1  # SSD
effective_io_concurrency = 200

# Logging
log_min_duration_statement = 1000  # Log queries > 1s
log_checkpoints = on
log_connections = on
log_disconnections = on
```

---

## Migration Strategy

### Option 1: Shadow Traffic (Gradual Migration)

**Approach**: Dual-write to both SQLite and PostgreSQL, gradually shift reads to PostgreSQL.

**Timeline**: 2-4 weeks

**Steps**:
1. **Week 1**: Setup PostgreSQL cluster, schema migration
2. **Week 2**: Dual-write (write to both SQLite + PostgreSQL), read from SQLite
3. **Week 3**: Validate data consistency, shift 50% reads to PostgreSQL
4. **Week 4**: Shift 100% reads to PostgreSQL, stop SQLite writes, decommission

**Pros**:
- Low risk (gradual rollout)
- Easy rollback (SQLite remains primary)
- Data validation in production environment

**Cons**:
- Requires code changes (dual-write logic)
- Longer timeline
- Increased infrastructure cost (running both databases)

---

### Option 2: Snapshot Export/Import (Downtime Migration)

**Approach**: Export SQLite data, import to PostgreSQL, switch over (brief downtime).

**Timeline**: 1-2 days (with 5-10 minutes downtime)

**Steps**:
1. **Day 1 (Preparation)**:
   - Setup PostgreSQL cluster
   - Apply schema migration
   - Test import on staging environment

2. **Day 2 (Cutover)**:
   - Schedule maintenance window (2 AM - 4 AM)
   - Stop backend servers (prevent new writes)
   - Export SQLite data (10-15 minutes for 10 GB database)
   - Import to PostgreSQL (5-10 minutes)
   - Update backend configuration (point to PostgreSQL)
   - Restart backend servers
   - Verify functionality

**Downtime**: 5-10 minutes (assuming 10 GB database)

**Pros**:
- Simple process
- No code changes required
- Fast completion (1-2 days)

**Cons**:
- Downtime required (5-10 minutes)
- Higher risk (no gradual rollout)
- Rollback requires restoring SQLite backup

---

### Recommended Approach: **Option 2 (Snapshot Export/Import)**

**Rationale**:
- Lower complexity (no dual-write code)
- Acceptable downtime (<10 minutes at 2 AM)
- Faster time to production
- Minimal infrastructure cost

---

## Schema Changes

### SQLite → PostgreSQL Mapping

| SQLite Type | PostgreSQL Type | Notes |
|-------------|----------------|-------|
| `TEXT` (UUID) | `UUID` | Native UUID type for better performance |
| `TEXT` (username, content) | `TEXT` | No change |
| `TEXT` (password_hash) | `VARCHAR(255)` | Fixed-length for bcrypt hashes |
| `INTEGER` (timestamps) | `BIGINT` | Millisecond UNIX timestamp |
| `BOOLEAN` | `BOOLEAN` | No change |
| `TIMESTAMP` | `TIMESTAMPTZ` | Timezone-aware timestamps (UTC) |

### Updated Schema (PostgreSQL)

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    password_salt VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    is_online BOOLEAN NOT NULL DEFAULT FALSE,
    last_seen_at TIMESTAMPTZ,
    CHECK (LENGTH(username) >= 1 AND LENGTH(username) <= 50)
);

CREATE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);
CREATE INDEX idx_users_is_online ON users(is_online) WHERE is_online = TRUE;

-- Conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user1_id UUID NOT NULL REFERENCES users(id),
    user2_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_message_at TIMESTAMPTZ,
    message_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE (user1_id, user2_id),
    CHECK (user1_id < user2_id),
    CHECK (user1_id != user2_id)
);

CREATE INDEX idx_conversations_user1_id ON conversations(user1_id);
CREATE INDEX idx_conversations_user2_id ON conversations(user2_id);
CREATE INDEX idx_conversations_updated_at ON conversations(updated_at DESC);

-- Messages table
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id),
    sender_id UUID NOT NULL REFERENCES users(id),
    recipient_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'delivered', 'failed')),
    is_anonymized BOOLEAN NOT NULL DEFAULT FALSE,
    CHECK (LENGTH(content) >= 1 AND LENGTH(content) <= 5000)
);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id, created_at DESC);
CREATE INDEX idx_messages_sender_id ON messages(sender_id, created_at DESC);
CREATE INDEX idx_messages_status ON messages(status) WHERE status IN ('pending', 'failed');
CREATE INDEX idx_messages_delivered_at ON messages(delivered_at) WHERE delivered_at IS NULL;

-- Full-text search (for message search feature)
ALTER TABLE messages ADD COLUMN content_tsvector tsvector;
CREATE INDEX idx_messages_fts ON messages USING gin(content_tsvector);

-- Trigger to update tsvector on INSERT/UPDATE
CREATE OR REPLACE FUNCTION messages_content_tsvector_update() RETURNS trigger AS $$
BEGIN
    NEW.content_tsvector := to_tsvector('english', NEW.content);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER messages_content_tsvector_trigger
BEFORE INSERT OR UPDATE ON messages
FOR EACH ROW EXECUTE FUNCTION messages_content_tsvector_update();
```

### Key Differences from SQLite

1. **UUID Type**: PostgreSQL has native UUID type (faster than TEXT)
2. **Timestamps**: PostgreSQL uses `TIMESTAMPTZ` (timezone-aware) instead of INTEGER milliseconds
3. **Full-Text Search**: PostgreSQL has built-in `tsvector` for efficient text search (vs. LIKE in SQLite)
4. **Indexes**: Added `idx_users_is_online` for presence queries
5. **Triggers**: Auto-update `content_tsvector` for search performance

---

## Data Migration Process

### Step 1: Export SQLite Data

```bash
#!/bin/bash
# export_sqlite.sh

DB_PATH="/var/lib/chat-server/chat.db"
EXPORT_DIR="/tmp/chat-migration"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$EXPORT_DIR"

# Export users
sqlite3 "$DB_PATH" <<EOF
.headers on
.mode csv
.output $EXPORT_DIR/users_$TIMESTAMP.csv
SELECT id, username, password_hash, password_salt,
       created_at, updated_at, deleted_at, is_online, last_seen_at
FROM users;
EOF

# Export conversations
sqlite3 "$DB_PATH" <<EOF
.headers on
.mode csv
.output $EXPORT_DIR/conversations_$TIMESTAMP.csv
SELECT id, user1_id, user2_id, created_at, updated_at, last_message_at, message_count
FROM conversations;
EOF

# Export messages
sqlite3 "$DB_PATH" <<EOF
.headers on
.mode csv
.output $EXPORT_DIR/messages_$TIMESTAMP.csv
SELECT id, conversation_id, sender_id, recipient_id, content,
       created_at, delivered_at, status, is_anonymized
FROM messages;
EOF

echo "Export completed: $EXPORT_DIR"
```

### Step 2: Data Transformation

```python
#!/usr/bin/env python3
# transform_data.py

import csv
import sys
from datetime import datetime

def transform_timestamp(milliseconds):
    """Convert milliseconds to ISO 8601 timestamp"""
    if milliseconds:
        return datetime.utcfromtimestamp(int(milliseconds) / 1000.0).isoformat() + 'Z'
    return None

def transform_users(input_csv, output_csv):
    with open(input_csv, 'r') as infile, open(output_csv, 'w') as outfile:
        reader = csv.DictReader(infile)
        writer = csv.DictWriter(outfile, fieldnames=reader.fieldnames)
        writer.writeheader()
        
        for row in reader:
            row['created_at'] = transform_timestamp(row['created_at'])
            row['updated_at'] = transform_timestamp(row['updated_at'])
            row['deleted_at'] = transform_timestamp(row['deleted_at']) if row['deleted_at'] else ''
            row['last_seen_at'] = transform_timestamp(row['last_seen_at']) if row['last_seen_at'] else ''
            writer.writerow(row)

# Similar functions for conversations and messages...

if __name__ == '__main__':
    transform_users(sys.argv[1], sys.argv[2])
```

### Step 3: Import to PostgreSQL

```bash
#!/bin/bash
# import_postgresql.sh

EXPORT_DIR="/tmp/chat-migration"
PG_HOST="localhost"
PG_PORT="5432"
PG_USER="chat_admin"
PG_DB="chat_db"

# Import users
psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB <<EOF
\COPY users FROM '$EXPORT_DIR/users_transformed.csv' WITH CSV HEADER;
EOF

# Import conversations
psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB <<EOF
\COPY conversations FROM '$EXPORT_DIR/conversations_transformed.csv' WITH CSV HEADER;
EOF

# Import messages
psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB <<EOF
\COPY messages FROM '$EXPORT_DIR/messages_transformed.csv' WITH CSV HEADER;
EOF

echo "Import completed"
```

### Step 4: Validation

```sql
-- Count validation
SELECT 'users' AS table_name, COUNT(*) AS count FROM users
UNION ALL
SELECT 'conversations', COUNT(*) FROM conversations
UNION ALL
SELECT 'messages', COUNT(*) FROM messages;

-- Data integrity checks
SELECT COUNT(*) AS orphaned_messages
FROM messages m
LEFT JOIN conversations c ON m.conversation_id = c.id
WHERE c.id IS NULL;

SELECT COUNT(*) AS invalid_users
FROM messages m
LEFT JOIN users u ON m.sender_id = u.id
WHERE u.id IS NULL;
```

---

## Testing & Validation

### Pre-Migration Tests (Staging)

1. **Schema Validation**
   - Apply schema migration on staging PostgreSQL
   - Verify all tables, indexes, constraints created
   - Verify foreign key relationships

2. **Data Migration Test**
   - Export staging SQLite data
   - Import to staging PostgreSQL
   - Verify row counts match
   - Verify data integrity (foreign keys, constraints)

3. **Performance Benchmarking**
   - Run query performance tests (compare SQLite vs. PostgreSQL)
   - Measure insert/update/select latency
   - Target: <10ms for simple queries, <100ms for complex queries

4. **Application Integration Test**
   - Update backend configuration (point to PostgreSQL)
   - Run full test suite (`cargo test --workspace`)
   - Run integration tests (E2E flow)

### Post-Migration Validation (Production)

1. **Smoke Tests** (Immediately After Cutover)
   - User signup/login works
   - Message send/receive works
   - Conversation history loads
   - User search works

2. **Health Checks**
   - Monitor `/health` endpoint
   - Check PostgreSQL connection pool status
   - Monitor error logs for database errors

3. **Performance Monitoring** (First 24 Hours)
   - Monitor query latency (p50, p95, p99)
   - Monitor message throughput
   - Monitor connection pool utilization
   - Monitor disk I/O

4. **Data Integrity Audit** (First Week)
   - Verify no data loss (compare row counts)
   - Verify foreign key constraints not violated
   - Verify message delivery status correct

---

## Rollback Plan

### Scenario: Critical Issue After Migration

**Trigger Conditions**:
- Data corruption detected
- Critical performance degradation (10x slower)
- Database unavailable (connection failures)

### Rollback Steps

1. **Stop Backend Servers**
   ```bash
   sudo systemctl stop chat-server
   ```

2. **Restore SQLite Backup**
   ```bash
   sudo cp /var/backups/chat-server/pre-migration-backup.db /var/lib/chat-server/chat.db
   sudo chown chat-server:chat-server /var/lib/chat-server/chat.db
   ```

3. **Update Backend Configuration**
   ```bash
   sudo nano /opt/chat-server/config.toml
   # Change:
   # [database]
   # type = "sqlite"
   # path = "/var/lib/chat-server/chat.db"
   ```

4. **Restart Backend Servers**
   ```bash
   sudo systemctl start chat-server
   ```

5. **Verify Functionality**
   ```bash
   curl http://localhost:8080/health
   ```

**Rollback Time**: 5-10 minutes

**Data Loss**: Any messages sent during PostgreSQL uptime are lost (recommend 30-minute maintenance window to minimize loss)

### Preventing Data Loss

- Keep PostgreSQL running for 24 hours post-migration (for forensics)
- Export PostgreSQL data before rollback
- Merge PostgreSQL changes into SQLite (manual recovery)

---

## Performance Considerations

### Expected Performance Gains

| Metric | SQLite (MVP) | PostgreSQL (Production) | Improvement |
|--------|-------------|------------------------|-------------|
| Write Throughput | ~100 msg/sec | ~1,000 msg/sec | **10x** |
| Read Latency (p50) | 10-50ms | 5-15ms | **2-3x faster** |
| Concurrent Connections | 10-50 | 200+ | **4-20x** |
| Database Size Limit | 10 GB (practical) | 1 TB+ | **100x** |
| Full-Text Search | LIKE (slow) | tsvector (fast) | **50-100x faster** |

### Optimization Tips

1. **Connection Pooling** (PgBouncer)
   - Transaction pooling mode (fastest)
   - Pool size: 100-200 connections
   - Max client connections: 1,000

2. **Indexes**
   - Add indexes on frequently queried columns (already in schema)
   - Use partial indexes (e.g., `WHERE is_online = TRUE`)
   - Monitor index usage with `pg_stat_user_indexes`

3. **Query Optimization**
   - Use `EXPLAIN ANALYZE` to identify slow queries
   - Add missing indexes if needed
   - Consider materialized views for aggregations

4. **Partitioning** (Future Optimization)
   - Partition `messages` table by `created_at` (monthly partitions)
   - Benefit: Faster queries on recent messages, easier archival

---

## Timeline & Resources

### Estimated Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| **Phase 1: Planning** | 1 week | Review schema, plan migration strategy |
| **Phase 2: Staging Setup** | 1 week | Setup PostgreSQL cluster, schema migration |
| **Phase 3: Testing** | 1 week | Data migration test, performance benchmarking |
| **Phase 4: Production Migration** | 1 day | Export, import, cutover (5-10 min downtime) |
| **Phase 5: Monitoring** | 1 week | Post-migration validation, performance tuning |

**Total Timeline**: 4-5 weeks

### Resource Requirements

**Infrastructure**:
- PostgreSQL Primary: 8 CPU, 32 GB RAM, 500 GB NVMe SSD ($200/month)
- PostgreSQL Standby: Same as primary ($200/month)
- PgBouncer (optional): Can run on backend servers ($0 additional)

**Personnel**:
- Database Administrator: 20 hours (schema migration, performance tuning)
- Backend Developer: 10 hours (configuration updates, testing)
- DevOps Engineer: 10 hours (infrastructure setup, monitoring)

**Total Cost**: ~$400/month infrastructure + ~$2,000 one-time labor

---

## Next Steps

1. **Schedule Migration** (Recommended: 3-6 months after MVP launch)
   - Monitor user growth
   - Trigger: User count > 1,000 or write throughput > 80 msg/sec

2. **Setup Staging Environment**
   - Install PostgreSQL 14+ on staging server
   - Apply schema migration
   - Test data migration

3. **Performance Benchmarking**
   - Run load tests on staging (compare SQLite vs. PostgreSQL)
   - Measure query latency, throughput
   - Validate 10x performance improvement

4. **Production Migration**
   - Schedule maintenance window (2 AM - 4 AM, off-peak)
   - Notify users 48 hours in advance
   - Execute migration (export → import → cutover)
   - Monitor for 24 hours post-migration

---

## References

- **PostgreSQL Documentation**: https://www.postgresql.org/docs/14/
- **sqlx Migration Guide**: https://docs.rs/sqlx/latest/sqlx/migrate/index.html
- **PgBouncer**: https://www.pgbouncer.org/
- **Data Model**: `specs/001-private-chat/data-model.md`
- **Deployment Guide**: `docs/DEPLOYMENT.md`

---

**Last Updated**: 2025-12-16  
**Maintained By**: Database Team  
**Status**: Ready for Execution
