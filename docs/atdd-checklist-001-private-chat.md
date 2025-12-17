# ATDD Checklist - Story 001-private-chat: Private Chat Application

**Date:** 2025-12-17
**Author:** Riddler
**Primary Test Level:** N/A (ATDD workflow halted)

---

## Status

**HALTED (preflight failure)**

This ATDD workflow requires an E2E framework scaffold (`playwright.config.*` or `cypress.config.*`) plus `package.json` dependencies. This repo is Rust-native and currently has neither.

**Evidence:**

- `package.json`: missing
- `playwright.config.*` / `cypress.config.*` / `cypress.json`: not found

---

## Story Summary

Implement a private 1:1 chat system with authentication, real-time messaging, offline queueing, message history/search, presence, and privacy guarantees.

---

## Acceptance Criteria (from `specs/001-private-chat/spec.md`)

1. **FR-001** Account creation with unique username + strong password
2. **FR-002** Authentication + secure JWT sessions
3. **FR-003** User search + start private chat
4. **FR-004** Send messages via WebSocket (<= 5000 chars)
5. **FR-005** Real-time delivery or offline queue with indefinite retry + exponential backoff
6. **FR-006** Preserve and display complete message history per conversation
7. **FR-007** Show timestamps and sender info
8. **FR-008** Presence updates within 1 second of login/logout
9. **FR-009** Search message history by keyword
10. **FR-010** Logout terminates session securely
11. **FR-011** Authorization: only participants can retrieve conversations; otherwise 403
12. **FR-012** Privacy/security requirements (bcrypt, JWT expiry, SQLite MVP, disk encryption prod)
13. **FR-013** Reject messages > 5000 chars with user-facing validation error
14. **FR-014** Account deletion; cannot be reactivated
15. **FR-015** Anonymize deleted-user messages (“Deleted User”)
16. **FR-016** Delivery status indicators (queued pending → delivered)
17. **FR-017** Auth rate limiting (5 failed attempts / IP / 15 minutes)

---

## What Would Be Generated (If Unblocked)

- Failing E2E/API/component tests implementing RED phase for the FR list above
- Factory + fixture scaffolding (faker, cleanup discipline, deterministic waits)
- Implementation checklist mapping tests → code tasks

---

## Options To Proceed

### Option A — Use this workflow as-is (Node E2E harness)

1. Add `package.json`
2. Run `*framework` to scaffold Playwright/Cypress
3. Re-run `*atdd` and generate RED tests

### Option B — Rust-native ATDD (recommended for this repo)

Generate failing acceptance tests as Rust integration tests (e.g., `src/backend/tests/*.rs`) using:

- `warp::test` for HTTP routes (403 authz, login rate limiting, logout, user search)
- In-memory/temporary SQLite for DB workflows (deletion + anonymization)
- `tokio-tungstenite` against an ephemeral server port for WS round-trips (send, oversize reject, delivery ack/status)

This aligns with the current test runner (`cargo test`) and avoids introducing Node tooling solely for test scaffolding.

---

## Related Evidence

- Traceability & gate: `docs/traceability-matrix.md`, `docs/gate-decision-story-001-private-chat.md`
- Test suite wiring gap: `docs/test-review.md`

