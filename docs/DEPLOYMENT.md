# Deployment Guide (MVP → Production)

## Storage & Encryption
- **MVP (Dev/Test)**: SQLite database stored in plaintext files (e.g., `chat.db`, WAL files)
- **Production**: Use full-disk encryption
  - Linux: LUKS (dm-crypt) with strong passphrase or hardware-backed TPM unlock
  - Windows: BitLocker or Windows Defender Device Encryption
- **Optional**: SQLCipher to encrypt SQLite (including WAL) when regulatory requirements demand file-level encryption
- **Backups**: Encrypted backups only; store encryption keys separately from backup media

## Operational Notes
- Keep database files (`*.db`, `*.db-wal`, `*.db-shm`) on encrypted volumes
- Restrict filesystem permissions to the service user running the chat server
- Rotate JWT secrets and restart the service when secrets change
- Monitor disk utilization; alert at 80% to prevent WAL growth issues

## Migration Path
- SQLite is the MVP storage engine; migration to PostgreSQL is documented separately (`docs/DEPLOYMENT_POSTGRES_MIGRATION.md`)
- Before migration, snapshot the encrypted volume and verify backups
