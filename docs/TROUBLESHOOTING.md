# Troubleshooting Guide: Private Chat Application

**Version**: 1.0.0  
**Last Updated**: 2025-12-16  

---

## Table of Contents

1. [Server Issues](#server-issues)
2. [WebSocket Connection Issues](#websocket-connection-issues)
3. [Authentication & Authorization](#authentication--authorization)
4. [Database Issues](#database-issues)
5. [Performance Issues](#performance-issues)
6. [Client Application Issues](#client-application-issues)
7. [Network & Firewall Issues](#network--firewall-issues)
8. [Logging & Debugging](#logging--debugging)

---

## Server Issues

### Server Won't Start

**Symptoms**:
- `systemctl start chat-server` fails
- Service exits immediately after start

**Common Causes & Solutions**:

1. **Port Already in Use**

```bash
# Check what's using port 8080
sudo lsof -i :8080
sudo netstat -tulpn | grep 8080

# Kill conflicting process
sudo kill -9 <PID>

# Or change port in config
sudo nano /opt/chat-server/config.toml
# Change: port = 8081
sudo systemctl restart chat-server
```

2. **Permission Denied (Database)**

```bash
# Check database file permissions
ls -la /var/lib/chat-server/

# Fix permissions
sudo chown -R chat-server:chat-server /var/lib/chat-server/
sudo chmod 700 /var/lib/chat-server/
sudo chmod 600 /var/lib/chat-server/*.db*
```

3. **Invalid Configuration**

```bash
# Check config syntax
cat /opt/chat-server/config.toml

# Validate JWT secret is set
grep jwt_secret /opt/chat-server/config.toml

# Generate new secret if needed
openssl rand -base64 32
```

4. **Missing Dependencies**

```bash
# Verify binary runs
/opt/chat-server/chat-server --version

# Check for missing libraries
ldd /opt/chat-server/chat-server

# Reinstall if needed
cargo build --release
sudo cp target/release/chat-server /opt/chat-server/
```

---

### Server Crashes or Restarts Frequently

**Symptoms**:
- Service restarts every few minutes
- Out of memory errors in logs

**Diagnosis**:

```bash
# Check system logs
sudo journalctl -u chat-server -n 100

# Check memory usage
free -h
ps aux | grep chat-server

# Check disk space
df -h /var/lib/chat-server
```

**Solutions**:

1. **Out of Memory**

```bash
# Increase system swap
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Or upgrade server RAM
```

2. **Disk Full**

```bash
# Clean up old logs
sudo journalctl --vacuum-time=7d

# Remove old database backups
sudo find /var/backups/chat-server -mtime +30 -delete

# Check database size
du -sh /var/lib/chat-server/
```

3. **Database Corruption**

```bash
# Stop server
sudo systemctl stop chat-server

# Check database integrity
sqlite3 /var/lib/chat-server/chat.db "PRAGMA integrity_check;"

# If corrupted, restore from backup
sudo cp /var/backups/chat-server/latest.db /var/lib/chat-server/chat.db
sudo chown chat-server:chat-server /var/lib/chat-server/chat.db

# Restart server
sudo systemctl start chat-server
```

---

## WebSocket Connection Issues

### Cannot Connect to WebSocket

**Symptoms**:
- Client shows "Connection failed" or "Cannot reach server"
- WebSocket upgrade fails with 400/401/403

**Common Causes**:

1. **Invalid or Expired Token**

```bash
# Test token validity
TOKEN="your-jwt-token-here"
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/user/me

# Expected: 200 OK with user data
# If 401: Token expired or invalid, user must re-login
```

**Solution**: User must log out and log back in to get a new token.

2. **WebSocket Upgrade Failure**

```bash
# Test WebSocket connection with websocat
websocat "ws://localhost:8080/socket?token=$TOKEN"

# Or with curl (check upgrade response)
curl -i -N -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Version: 13" \
     -H "Sec-WebSocket-Key: $(openssl rand -base64 16)" \
     "http://localhost:8080/socket?token=$TOKEN"
```

**Expected Response**:
```
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
```

3. **Firewall Blocking WebSocket**

```bash
# Check firewall rules
sudo ufw status

# Allow WebSocket port
sudo ufw allow 8080/tcp

# Or if behind nginx proxy
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

4. **Reverse Proxy Misconfiguration**

Check nginx config for WebSocket support:

```nginx
location /socket {
    proxy_pass http://localhost:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    
    # Increase timeouts for long-lived connections
    proxy_read_timeout 300s;
    proxy_send_timeout 300s;
}
```

Reload nginx:
```bash
sudo nginx -t
sudo systemctl reload nginx
```

---

### WebSocket Connection Drops Frequently

**Symptoms**:
- Client disconnects every 30-60 seconds
- "Connection lost" messages

**Diagnosis**:

```bash
# Check server logs for heartbeat timeouts
sudo journalctl -u chat-server | grep -i "heartbeat\|ping\|pong"

# Check network stability
ping -c 100 chat.example.com
```

**Solutions**:

1. **Heartbeat Timeout Too Aggressive**

Server sends PING every 25s, expects PONG within 5s. If client doesn't respond, connection closes.

**Solution**: Ensure client WebSocket library handles PING/PONG automatically (most do by default).

2. **Network Instability**

```bash
# Test connection quality
mtr -r -c 100 chat.example.com

# Check for packet loss > 1%
```

**Solution**: Improve network infrastructure or implement exponential backoff reconnection on client.

3. **Load Balancer Idle Timeout**

If using load balancer, ensure idle timeout > 60s:

```nginx
# In nginx upstream block
keepalive_timeout 300s;
```

---

## Authentication & Authorization

### Cannot Log In

**Symptoms**:
- Login returns 401 Unauthorized
- "Invalid credentials" error

**Common Causes**:

1. **Incorrect Password**

```bash
# Verify username exists
sqlite3 /var/lib/chat-server/chat.db "SELECT username FROM users WHERE username='alice';"

# Check if account deleted
sqlite3 /var/lib/chat-server/chat.db "SELECT deleted_at FROM users WHERE username='alice';"

# If deleted_at is NOT NULL, account is deleted and cannot log in
```

2. **Rate Limiting Triggered**

```bash
# Check failed login attempts
sudo journalctl -u chat-server | grep "failed login" | tail -20

# Wait 15 minutes for rate limit to reset
# Or manually reset (requires database access):
sqlite3 /var/lib/chat-server/chat.db "DELETE FROM auth_logs WHERE username='alice' AND created_at < datetime('now', '-15 minutes');"
```

3. **Account Locked or Deleted**

**Solution**: User must create a new account if account was deleted (no reactivation).

---

### Token Expired Mid-Session

**Symptoms**:
- Client shows "Session expired" after 1 hour
- 401 Unauthorized on API calls

**Solution**:

Implement automatic token refresh in client:

```javascript
// Client-side: Refresh token before expiration
setInterval(async () => {
  const response = await fetch('http://localhost:8080/auth/refresh', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${currentToken}`
    }
  });
  
  if (response.ok) {
    const { token } = await response.json();
    localStorage.setItem('token', token);
  }
}, 50 * 60 * 1000); // Refresh every 50 minutes (before 1-hour expiry)
```

---

## Database Issues

### Database Locked

**Symptoms**:
- Error: "database is locked"
- Operations timeout

**Common Causes**:

1. **Multiple Processes Accessing Database**

```bash
# Check for multiple server instances
ps aux | grep chat-server

# Kill duplicate instances
sudo systemctl restart chat-server
```

2. **Unfinished Transactions**

```bash
# Check for WAL file
ls -lh /var/lib/chat-server/*.db-wal

# Force WAL checkpoint
sqlite3 /var/lib/chat-server/chat.db "PRAGMA wal_checkpoint(TRUNCATE);"
```

3. **File System Issues**

```bash
# Check disk errors
dmesg | grep -i error

# Check file system
sudo fsck /dev/sda1  # Replace with correct partition
```

**Solution**:

```bash
# Stop server
sudo systemctl stop chat-server

# Remove lock files
sudo rm /var/lib/chat-server/*.db-shm
sudo rm /var/lib/chat-server/*.db-wal

# Restart server
sudo systemctl start chat-server
```

---

### Database Corruption

**Symptoms**:
- Integrity check fails
- Queries return unexpected results
- Server crashes on startup

**Diagnosis**:

```bash
# Check database integrity
sqlite3 /var/lib/chat-server/chat.db "PRAGMA integrity_check;"

# Expected: "ok"
# If errors, database is corrupted
```

**Recovery**:

```bash
# Stop server
sudo systemctl stop chat-server

# Backup corrupted database
sudo cp /var/lib/chat-server/chat.db /var/lib/chat-server/chat.db.corrupted

# Option 1: Restore from backup
sudo cp /var/backups/chat-server/latest.db /var/lib/chat-server/chat.db

# Option 2: Attempt repair (not always successful)
sqlite3 /var/lib/chat-server/chat.db.corrupted ".recover" | sqlite3 /var/lib/chat-server/chat.db

# Fix permissions
sudo chown chat-server:chat-server /var/lib/chat-server/chat.db

# Restart server
sudo systemctl start chat-server
```

---

## Performance Issues

### High CPU Usage

**Symptoms**:
- Server uses > 80% CPU constantly
- API responses slow

**Diagnosis**:

```bash
# Check CPU usage
top -p $(pgrep chat-server)

# Profile with perf (requires debug symbols)
sudo perf record -p $(pgrep chat-server) -g -- sleep 10
sudo perf report
```

**Common Causes & Solutions**:

1. **Message Flood (DDoS or Bug)**

```bash
# Check active connections
curl -s http://localhost:8080/status | jq '.connections.activeWebsocket'

# Check message rate
sudo journalctl -u chat-server | grep "message sent" | tail -100 | wc -l

# If abnormally high, enable rate limiting or block IP
sudo ufw deny from <ATTACKER_IP>
```

2. **Inefficient Queries**

```bash
# Enable query logging (if not enabled)
sqlite3 /var/lib/chat-server/chat.db "PRAGMA query_only = ON;"

# Check slow queries in logs
sudo journalctl -u chat-server | grep "slow query"
```

**Solution**: Add database indexes (should be already applied in schema).

---

### High Memory Usage

**Symptoms**:
- Server uses > 1GB RAM
- System swapping heavily

**Diagnosis**:

```bash
# Check memory usage
ps aux | grep chat-server

# Check for memory leaks
sudo valgrind --leak-check=full /opt/chat-server/chat-server --config /opt/chat-server/config.toml
```

**Solutions**:

1. **Too Many Connections**

```bash
# Limit max connections (in future version)
# For now, use nginx to limit concurrent connections

# In nginx config:
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;
limit_conn conn_limit 10;  # Max 10 connections per IP
```

2. **Large Message History Loads**

**Solution**: Implement pagination on client side (load 50 messages at a time, not entire history).

---

### Slow API Responses

**Symptoms**:
- API calls take > 1 second
- Timeouts

**Diagnosis**:

```bash
# Measure response time
time curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/user/me

# Check database query time
sqlite3 /var/lib/chat-server/chat.db "EXPLAIN QUERY PLAN SELECT * FROM messages WHERE conversation_id='conv-123';"
```

**Solutions**:

1. **Missing Indexes**

Verify indexes exist:
```bash
sqlite3 /var/lib/chat-server/chat.db ".indexes"
```

Expected indexes:
- `idx_users_username`
- `idx_conversations_user1_id`
- `idx_conversations_user2_id`
- `idx_messages_conversation_id`

2. **Database Too Large**

```bash
# Check database size
du -sh /var/lib/chat-server/chat.db

# If > 10 GB, consider migrating to PostgreSQL
```

See `DEPLOYMENT.md` for PostgreSQL migration guide.

---

## Client Application Issues

### Client Won't Start

**Windows Client**:

1. **Missing Runtime Dependencies**

```powershell
# Check for missing DLLs
dumpbin /dependents chat-client.exe

# Install Visual C++ Redistributable
# Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe
```

2. **Antivirus Blocking**

Some antivirus software blocks unsigned executables.

**Solution**: Add exception for `chat-client.exe` in antivirus settings, or sign the executable.

---

### Client Cannot Connect to Server

**Common Causes**:

1. **Server URL Incorrect**

Check client configuration (if configurable) or hardcoded server URL.

2. **Certificate Validation Failure (WSS)**

If using self-signed certificates:

```rust
// In client code, disable certificate validation (dev only!)
let connector = TlsConnector::builder()
    .danger_accept_invalid_certs(true)
    .build()?;
```

**For production**: Use Let's Encrypt certificates.

3. **Firewall Blocking Outbound Connections**

```bash
# Test outbound connection
telnet chat.example.com 443

# Or with curl
curl -I https://chat.example.com/health
```

---

## Network & Firewall Issues

### Cannot Access Server from External Network

**Symptoms**:
- Server works on localhost but not from external IP
- Connection timeout from internet

**Common Causes**:

1. **Firewall Blocking**

```bash
# Check firewall status
sudo ufw status

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Reload firewall
sudo ufw reload
```

2. **Server Bound to Localhost Only**

Check config:
```toml
[server]
host = "0.0.0.0"  # Listen on all interfaces
# NOT: host = "127.0.0.1"  # Localhost only
```

3. **Port Forwarding Not Configured (NAT)**

If server is behind NAT router, configure port forwarding:
- External port 443 → Internal IP:443

---

### CORS Errors in Browser

**Symptoms**:
- Browser console shows "CORS policy blocked"
- Fetch requests fail

**Solution**:

Verify CORS headers in nginx config:

```nginx
location / {
    # Add CORS headers
    add_header 'Access-Control-Allow-Origin' '*' always;  # Or specific domain
    add_header 'Access-Control-Allow-Methods' 'GET, POST, DELETE, OPTIONS' always;
    add_header 'Access-Control-Allow-Headers' 'Authorization, Content-Type' always;
    
    if ($request_method = 'OPTIONS') {
        return 204;
    }
    
    proxy_pass http://localhost:8080;
}
```

Reload nginx:
```bash
sudo nginx -t
sudo systemctl reload nginx
```

---

## Logging & Debugging

### Enable Debug Logging

```bash
# Edit config
sudo nano /opt/chat-server/config.toml

# Change log level
[server]
log_level = "debug"  # Or "trace" for maximum verbosity

# Restart server
sudo systemctl restart chat-server

# View logs
sudo journalctl -u chat-server -f
```

---

### Export Logs for Support

```bash
# Export last 1000 log lines
sudo journalctl -u chat-server -n 1000 --no-pager > /tmp/chat-server-logs.txt

# Include system info
uname -a >> /tmp/chat-server-logs.txt
free -h >> /tmp/chat-server-logs.txt
df -h >> /tmp/chat-server-logs.txt

# Compress and send
tar -czf chat-server-debug-$(date +%Y%m%d).tar.gz /tmp/chat-server-logs.txt
```

---

### Run Server in Foreground (Debug Mode)

```bash
# Stop systemd service
sudo systemctl stop chat-server

# Run manually with debug output
sudo -u chat-server /opt/chat-server/chat-server --config /opt/chat-server/config.toml --log-level trace

# Press Ctrl+C to stop
# Restart service when done
sudo systemctl start chat-server
```

---

## Getting Help

If issues persist after following this guide:

1. **Check GitHub Issues**: https://github.com/your-org/chat-app/issues
2. **Community Forum**: Discord or Slack channel
3. **Commercial Support**: support@example.com

**When reporting issues, include**:
- Server version (`/opt/chat-server/chat-server --version`)
- OS and version (`uname -a`)
- Log excerpt (last 100 lines)
- Steps to reproduce
- Expected vs. actual behavior

---

**Last Updated**: 2025-12-16  
**Maintained By**: Support Team
