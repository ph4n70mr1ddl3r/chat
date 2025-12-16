# Privacy Policy (MVP)

## Overview
This self-hosted chat application stores all data locally on the server you control. No telemetry, analytics, or third-party data sharing is implemented.

## Data Collected
- Usernames
- Password hashes (bcrypt; salts stored alongside hashes)
- Message content and timestamps
- Presence metadata (online/offline status, last seen)
- Authentication tokens (JWT, in memory/on disk only as chosen by deployer)

## Data Retention
- Messages: retained indefinitely for conversation history
- Accounts: soft-deleted by setting `deleted_at`; records are not purged
- Presence: transient; recalculated on server startup

## Data Deletion and Anonymization
- Account deletion marks the user as deleted (`deleted_at`) and flags their messages as anonymized (`is_anonymized = TRUE`)
- UI displays messages from deleted accounts as "Deleted User"; message content remains for conversation continuity
- Deleted accounts cannot be reactivated

## Right to Be Forgotten (Anonymization)
- Implemented via anonymization rather than physical deletion
- Messages remain for historical integrity; sender identity is removed from display

## Data Sharing
- No data is sent to external services
- No analytics, telemetry, or advertising integrations are present

## Security Notes
- Authentication via JWT; tokens validated on every request/handshake
- Passwords are never stored in plaintext; bcrypt with salts is required
- Input validation and rate limiting are enforced on auth and messaging endpoints

## Login Notice (UI)
- The login screen displays: "This is a self-hosted chat application. Your data is stored locally and never shared externally."

## Encryption Guidance (Production)
- MVP uses plaintext SQLite files for local development
- Production deployments should use full-disk encryption (LUKS on Linux, BitLocker on Windows)
- Optional: integrate SQLCipher for SQLite WAL encryption if regulatory requirements demand it

## Contact
- For questions or requests, contact the deployment administrator responsible for the self-hosted server
