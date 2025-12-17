# Quality Gate Decision - Story 001-private-chat

**Story:** Private Chat Application (`001-private-chat`)
**Date:** 2025-12-17
**Evaluator:** TEA (Codex CLI)
**Decision Mode:** Deterministic

---

## Decision

**❌ FAIL (BLOCKED)**

---

## Evidence

### Traceability Results

Source: `docs/traceability-matrix.md`

- **P0 FULL coverage:** 1/10 (10%) → **fails** (requires 100%)
- **P1 FULL coverage:** 0/7 (0%) → **fails** (recommends 90%)
- **Overall FULL coverage:** 1/17 (6%) → **fails** (recommends 80%)

### Test Execution Results

Command: `cargo test`

Observed results (local):

- Total: 149 passed, 1 ignored, 0 failed
  - `chat_backend`: 136 passed, 1 ignored
  - `chat_backend` integration test `src/backend/tests/tokens_test.rs`: 10 passed
  - `chat_gui`: 3 passed

Ignored tests:

- `src/backend/server.rs:830` (`test_auth_rate_limit_blocks_after_failures`) is `#[ignore]`

---

## Rationale (Why Blocked)

1. **P0 requirements lack FULL executed coverage**, notably:
   - FR-011 authorization enforcement (403) has no executed tests.
   - FR-014 account deletion + FR-015 anonymization have no executed tests.
   - FR-005 offline retry/backoff is not validated by executed tests.
   - FR-017 auth endpoint rate limiting is not validated end-to-end (ignored route test).
2. **Majority of the story-aligned tests live under `tests/**` but are not executed by `cargo test`**, so they cannot be used as release evidence without wiring.
3. **Note:** FR-013 message length rejection is now FULL at the WebSocket boundary (see `src/backend/handlers/dispatcher.rs:249`), but it does not change the overall gate outcome.

---

## Required Remediation (To Reach PASS)

1. **Make the story test suite executable**
   - Option A: Move relevant `tests/**` into crate-level integration tests under `src/backend/tests/` (or per-crate `tests/` dirs).
   - Option B: Add a dedicated workspace member (e.g., `chat-tests`) with its own `Cargo.toml` that builds/runs the `tests/**` modules.
2. **Close P0 coverage gaps with executed tests**
   - Authorization: non-participant access returns `403 Forbidden`.
   - Deletion + anonymization: deleted users cannot be reactivated; messages display “Deleted User”.
   - Offline retry/backoff: deterministic tests for retry loop and state transitions.
   - Auth rate limiting: un-ignore and make endpoint testable with real IP extraction + limiter wiring.

---

## Optional Waiver

No waiver recorded. If business requires release with known gaps, document the waiver explicitly (scope, timebox, and compensating controls).
