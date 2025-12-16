# Deployment Guide: Private Chat Application

**Version**: 1.0.0  
**Last Updated**: 2025-12-16  
**Target Platform**: Linux (Ubuntu 20.04+) / Windows Server  

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [MVP Deployment (Single Server)](#mvp-deployment-single-server)
4. [Production Deployment](#production-deployment)
5. [Security Hardening](#security-hardening)
6. [Monitoring & Observability](#monitoring--observability)
7. [Backup & Recovery](#backup--recovery)
8. [Scaling & Performance](#scaling--performance)
9. [Migration Path (SQLite → PostgreSQL)](#migration-path-sqlite--postgresql)

---

## Overview

This guide covers deployment strategies from MVP (single-server SQLite) to production-ready (multi-node PostgreSQL with load balancing).

### Deployment Tiers

| Tier | Platform | Database | Users | Uptime Target |
|------|----------|----------|-------|---------------|
| **Development** | Local/WSL | SQLite | 1-10 | N/A |
| **MVP** | Single Linux Server | SQLite | 100-1,000 | 95% |
| **Production** | Multi-Node Cluster | PostgreSQL | 1,000-10,000 | 99% |
| **Enterprise** | Cloud/K8s | PostgreSQL + Redis | 10,000+ | 99.9% |

---

## Prerequisites

### System Requirements

**Minimum (MVP)**:
- **OS**: Ubuntu 20.04+ or Windows Server 2019+
- **CPU**: 2 cores (4 recommended)
- **RAM**: 4 GB (8 GB recommended)
- **Disk**: 50 GB SSD (IOPS: 3000+ recommended)
- **Network**: 100 Mbps (1 Gbps for production)

**Production**:
- **OS**: Ubuntu 22.04 LTS
- **CPU**: 4-8 cores per node
- **RAM**: 16-32 GB per node
- **Disk**: 100+ GB SSD (NVMe recommended)
- **Network**: 10 Gbps internal, 1 Gbps external

### Software Dependencies

- **Rust**: 1.75+ (stable)
- **SQLite**: 3.35+ (MVP) or **PostgreSQL**: 14+ (production)
- **systemd**: For service management (Linux)
- **nginx/Caddy**: Reverse proxy with TLS termination
- **Let's Encrypt**: Free TLS certificates

---

## MVP Deployment (Single Server)

### Step 1: Build the Application

```bash
# Clone repository
git clone https://github.com/your-org/chat-app.git
cd chat-app

# Build release binaries
cargo build --release --workspace

# Binaries output:
# - target/release/chat-server (backend)
# - target/release/chat-client (frontend, distribute separately)
```

### Step 2: Create Service User

```bash
# Create dedicated user for security
sudo useradd -r -s /bin/false chat-server
sudo mkdir -p /opt/chat-server
sudo chown chat-server:chat-server /opt/chat-server
```

### Step 3: Deploy Server Binary

```bash
# Copy binary
sudo cp target/release/chat-server /opt/chat-server/
sudo chown chat-server:chat-server /opt/chat-server/chat-server
sudo chmod 755 /opt/chat-server/chat-server

# Create data directory
sudo mkdir -p /var/lib/chat-server
sudo chown chat-server:chat-server /var/lib/chat-server
sudo chmod 700 /var/lib/chat-server
```

### Step 4: Configuration

Create `/opt/chat-server/config.toml`:

```toml
[server]
port = 8080
host = "0.0.0.0"
log_level = "info"

[database]
type = "sqlite"
path = "/var/lib/chat-server/chat.db"

[auth]
jwt_secret = "CHANGE_THIS_SECRET_KEY_IN_PRODUCTION"
token_expiry_seconds = 3600

[rate_limit]
enabled = true
max_requests_per_minute = 1000
max_auth_attempts = 5
```

**Security**: Generate JWT secret:
```bash
openssl rand -base64 32
```

### Step 5: Create systemd Service

Create `/etc/systemd/system/chat-server.service`:

```ini
[Unit]
Description=Private Chat Application Server
After=network.target

[Service]
Type=simple
User=chat-server
Group=chat-server
WorkingDirectory=/opt/chat-server
ExecStart=/opt/chat-server/chat-server --config /opt/chat-server/config.toml
Restart=always
RestartSec=5s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/chat-server

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=chat-server

[Install]
WantedBy=multi-user.target
```

### Step 6: Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable chat-server
sudo systemctl start chat-server

# Check status
sudo systemctl status chat-server

# View logs
sudo journalctl -u chat-server -f
```

### Step 7: Configure Firewall

```bash
# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow WebSocket port (if not behind reverse proxy)
sudo ufw allow 8080/tcp

# Enable firewall
sudo ufw enable
```

### Step 8: Setup Reverse Proxy (nginx)

Install nginx:
```bash
sudo apt update
sudo apt install nginx certbot python3-certbot-nginx
```

Create `/etc/nginx/sites-available/chat-app`:

```nginx
upstream chat_backend {
    server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name chat.example.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name chat.example.com;

    # TLS certificates (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/chat.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/chat.example.com/privkey.pem;

    # TLS security
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # WebSocket upgrade
    location /socket {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket timeouts
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
    }

    # REST API endpoints
    location / {
        proxy_pass http://chat_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check
    location /health {
        proxy_pass http://chat_backend;
        access_log off;
    }
}
```

Enable site and obtain certificate:

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/chat-app /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Obtain Let's Encrypt certificate
sudo certbot --nginx -d chat.example.com
```

### Step 9: Verify Deployment

```bash
# Test health endpoint
curl https://chat.example.com/health

# Expected response:
# {"status":"healthy","timestamp":1702657890000,"uptime_seconds":120}

# Test WebSocket connection
websocat "wss://chat.example.com/socket?token=<JWT_TOKEN>"
```

---

## Production Deployment

### Architecture Overview

```
                    ┌─────────────┐
                    │   Clients   │
                    └─────┬───────┘
                          │
                    ┌─────▼───────┐
                    │ Load Balancer│
                    │   (nginx)    │
                    └─────┬───────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
    ┌─────▼─────┐   ┌────▼─────┐   ┌────▼─────┐
    │  Server 1  │   │ Server 2 │   │ Server 3 │
    └─────┬─────┘   └────┬─────┘   └────┬─────┘
          │               │               │
          └───────────────┼───────────────┘
                          │
                    ┌─────▼──────┐
                    │ PostgreSQL │
                    │  (Primary) │
                    └─────┬──────┘
                          │
                    ┌─────▼──────┐
                    │ PostgreSQL │
                    │  (Replica) │
                    └────────────┘
```

### Load Balancer Configuration

Use nginx with session affinity (sticky sessions) for WebSocket connections:

```nginx
upstream chat_cluster {
    ip_hash;  # Sticky sessions based on client IP
    server 10.0.1.10:8080;
    server 10.0.1.11:8080;
    server 10.0.1.12:8080;
}

server {
    listen 443 ssl http2;
    server_name chat.example.com;

    # ... (same TLS config as MVP)

    location / {
        proxy_pass http://chat_cluster;
        # ... (same proxy settings)
    }
}
```

### PostgreSQL Migration

See [`DEPLOYMENT_POSTGRES_MIGRATION.md`](./DEPLOYMENT_POSTGRES_MIGRATION.md) for detailed migration steps.

---

## Security Hardening

### Storage & Encryption

**MVP (Dev/Test)**: 
- SQLite database stored in plaintext files (e.g., `chat.db`, WAL files)

**Production**: 
- Use full-disk encryption:
  - **Linux**: LUKS (dm-crypt) with strong passphrase or hardware-backed TPM unlock
  - **Windows**: BitLocker or Windows Defender Device Encryption
- **Optional**: SQLCipher to encrypt SQLite (including WAL) when regulatory requirements demand file-level encryption
- **Backups**: Encrypted backups only; store encryption keys separately from backup media

### Filesystem Permissions

```bash
# Restrict database files
sudo chown chat-server:chat-server /var/lib/chat-server/*.db*
sudo chmod 600 /var/lib/chat-server/*.db*

# Restrict config file
sudo chmod 600 /opt/chat-server/config.toml
```

### JWT Secret Rotation

```bash
# Generate new secret
NEW_SECRET=$(openssl rand -base64 32)

# Update config
sudo sed -i "s/jwt_secret = .*/jwt_secret = \"$NEW_SECRET\"/" /opt/chat-server/config.toml

# Restart service
sudo systemctl restart chat-server

# Note: All existing tokens will be invalidated; users must re-login
```

### TLS Best Practices

- Use TLS 1.2+ only (disable TLS 1.0/1.1)
- Enable HSTS (Strict-Transport-Security)
- Obtain certificates from Let's Encrypt (free, auto-renewal)
- Monitor certificate expiration (certbot auto-renews)

---

## Monitoring & Observability

### Logs

**Location**: 
- systemd journal: `/var/log/journal/`
- Application logs: `/var/lib/chat-server/logs/` (if configured)

**View logs**:
```bash
# Real-time logs
sudo journalctl -u chat-server -f

# Last 100 lines
sudo journalctl -u chat-server -n 100

# Filter by date
sudo journalctl -u chat-server --since "2025-12-16" --until "2025-12-17"
```

### Metrics

**Basic metrics**:
```bash
# Active connections
curl -s http://localhost:8080/status | jq '.connections'

# Memory usage
ps aux | grep chat-server

# Disk usage
df -h /var/lib/chat-server
```

**Production monitoring** (optional):
- **Prometheus**: Scrape `/metrics` endpoint
- **Grafana**: Visualize metrics
- **Alertmanager**: Configure alerts for downtime, high latency, disk usage

### Health Checks

Configure load balancer health check:

```nginx
location /health {
    proxy_pass http://chat_backend;
    access_log off;
    
    # Mark backend as down if health check fails
    proxy_next_upstream error timeout invalid_header http_500 http_502 http_503;
}
```

---

## Backup & Recovery

### Backup Strategy

**SQLite (MVP)**:

```bash
# Automated backup script
#!/bin/bash
BACKUP_DIR="/var/backups/chat-server"
DB_PATH="/var/lib/chat-server/chat.db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create backup with WAL checkpoint
sqlite3 "$DB_PATH" "PRAGMA wal_checkpoint(FULL);"
cp "$DB_PATH" "$BACKUP_DIR/chat_$TIMESTAMP.db"

# Encrypt backup
gpg --encrypt --recipient backup@example.com "$BACKUP_DIR/chat_$TIMESTAMP.db"
rm "$BACKUP_DIR/chat_$TIMESTAMP.db"

# Retain last 7 days
find "$BACKUP_DIR" -name "chat_*.db.gpg" -mtime +7 -delete
```

**Schedule with cron**:
```bash
# Add to /etc/cron.d/chat-backup
0 2 * * * chat-server /opt/chat-server/backup.sh
```

### Recovery

**Restore from backup**:
```bash
# Stop service
sudo systemctl stop chat-server

# Decrypt and restore
gpg --decrypt /var/backups/chat-server/chat_20251216_020000.db.gpg > /tmp/chat.db
sudo cp /tmp/chat.db /var/lib/chat-server/chat.db
sudo chown chat-server:chat-server /var/lib/chat-server/chat.db

# Start service
sudo systemctl start chat-server
```

---

## Scaling & Performance

### Vertical Scaling (Single Server)

Increase resources:
- CPU: 2 → 4 → 8 cores
- RAM: 4 → 8 → 16 GB
- Disk: Standard SSD → NVMe SSD

### Horizontal Scaling (Multi-Node)

1. Deploy multiple server instances
2. Configure load balancer (nginx with `ip_hash`)
3. Migrate to PostgreSQL (shared state)
4. Add Redis pub/sub for WebSocket message distribution

### Performance Tuning

**SQLite**:
```sql
-- Enable WAL mode
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64 MB cache
PRAGMA temp_store = MEMORY;
```

**PostgreSQL** (production):
```sql
-- Connection pooling
max_connections = 200
shared_buffers = 4GB
effective_cache_size = 12GB
work_mem = 64MB
maintenance_work_mem = 1GB
```

---

## Operational Notes

### Daily Operations

- Keep database files (`*.db`, `*.db-wal`, `*.db-shm`) on encrypted volumes
- Restrict filesystem permissions to the service user running the chat server
- Rotate JWT secrets and restart the service when secrets change
- Monitor disk utilization; alert at 80% to prevent WAL growth issues

### Maintenance Windows

- Schedule downtime for updates: Off-peak hours (e.g., 2-4 AM local time)
- Test updates on staging environment first
- Keep rollback plan ready (restore from backup)

---

## Migration Path (SQLite → PostgreSQL)

SQLite is the MVP storage engine; migration to PostgreSQL is documented separately in [`DEPLOYMENT_POSTGRES_MIGRATION.md`](./DEPLOYMENT_POSTGRES_MIGRATION.md).

**Before migration**:
- Snapshot the encrypted volume
- Verify backups
- Test migration on staging environment

---

## Troubleshooting

See [`TROUBLESHOOTING.md`](./TROUBLESHOOTING.md) for common issues and solutions.

---

## Support & Resources

- **GitHub Issues**: Report bugs and feature requests
- **Documentation**: `docs/` directory
- **Community**: Discord or Slack channel
- **Commercial Support**: Contact support@example.com

---

**Last Updated**: 2025-12-16  
**Maintained By**: DevOps Team
