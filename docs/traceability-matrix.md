# Traceability Matrix & Quality Gate - Story 001-private-chat

**Story:** Private Chat Application (`001-private-chat`)
**Date:** 2025-12-17
**Evaluator:** TEA (Codex CLI)

---

## PHASE 1: REQUIREMENTS TRACEABILITY

### Test Suite Discovery Notes

- `tests/` contains many BDD-style Rust files with `Test ID:` + Given/When/Then, but it is **not executed by `cargo test`** because the workspace root has no crate and `tests/` is not a workspace member.
- Executed automated tests are primarily **unit/integration tests embedded in** `src/backend/**` and `src/frontend/**` (`#[test]` / `#[tokio::test]`), but they mostly **lack Test IDs** and BDD structure.
- Note: `cargo test` executes per-crate integration tests under `src/backend/tests/**` (e.g., `src/backend/tests/tokens_test.rs`), which are distinct from the workspace-root `tests/**` directory.

This matrix prioritizes **executed automated tests** for coverage status and gate metrics. `tests/**` artifacts are listed as supplemental evidence only (quality and wiring gaps are tracked below).

---

### Test Execution Evidence

Command: `cargo test`

Observed results (local):

- `chat_backend`: 136 passed, 1 ignored
- `chat_backend` integration test `src/backend/tests/tokens_test.rs`: 10 passed
- `chat_gui`: 3 passed

---

### Coverage Summary (Executed Automated Tests)

| Priority  | Total Criteria | FULL Coverage | Coverage % | Status |
| --------- | -------------- | ------------- | ---------- | ------ |
| P0        | 10             | 1             | 10%        | ❌ FAIL |
| P1        | 7              | 0             | 0%         | ❌ FAIL |
| P2        | 0              | 0             | N/A        | N/A    |
| P3        | 0              | 0             | N/A        | N/A    |
| **Total** | **17**         | **1**         | **6%**     | **❌ FAIL** |

**Gate thresholds (deterministic):**

- P0 coverage >= 100% (required)
- P1 coverage >= 90% (recommended)
- Overall coverage >= 80% (recommended)

---

### Detailed Mapping

#### FR-001: Account creation with unique username + strong password (P0)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit):**
  - `src/backend/validators/mod.rs:104` (`test_validate_password_valid`) – validates password rules.
  - `src/backend/services/auth_service.rs:246` (`test_create_user`) – creates user and hashes password.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/e2e_test.rs:240` (`T100-002`) – signup validation + duplicate username (requires wiring into a runnable test crate).
- **Gaps:**
  - Unique username constraint (DB uniqueness + handler-level error) not verified in executed tests.
  - End-to-end signup handler behavior not verified in executed tests.
- **Recommendation:** Add API/handler tests for `/auth/signup` covering duplicate username and password validation errors.

---

#### FR-002: Authenticate + maintain secure sessions using JWT (P0)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit):**
  - `src/backend/services/auth_service.rs:259` (`test_generate_token`) – token created with future expiration.
  - `src/backend/services/auth_service.rs:270` (`test_verify_token_valid`) – token verification success.
  - `src/backend/middleware/auth.rs:61` (`test_extract_user_id_valid`) – extracts user id from `Authorization: Bearer ...`.
  - `src/backend/handlers/handshake.rs:171` (`test_handshake_validator_valid_token`) – validates WebSocket upgrade token.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/tokens_integration_test.rs:18` (`T200-001`+) – token scenarios (requires wiring).
- **Gaps:**
  - Login endpoint (`/auth/login`) not directly tested in executed suite.
  - Token expiration edge cases (expired token handling) not covered in executed suite at HTTP boundary.
- **Recommendation:** Add `warp::test` route tests for `/auth/login` success/failure and expired token rejection on protected endpoints.

---

#### FR-003: Search/select other users to start private chat (P1)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit/DB):**
  - `src/backend/db/queries/mod.rs:537` (`test_search_users_is_safe_against_sql_injection`) – search query is parameterized/safe.
  - `src/backend/services/user_service.rs:133` (`search_results_are_cached_until_ttl_expires`) – caching wrapper behavior.
  - `src/backend/services/conversation_service.rs:111` (`test_create_conversation`) – creates one-to-one conversation and enforces ordering.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/user_search_test.rs:11` (`T003-001`+) – user search scenarios (requires wiring).
  - `tests/integration/conversation_test.rs:11` (`T050-001`+) – conversation start constraints (requires wiring).
- **Gaps:**
  - `/users/search` endpoint behavior (excludes self, pagination) not tested in executed suite.
  - Handler-level authorization for starting conversation not tested in executed suite.
- **Recommendation:** Add handler/API tests for `/users/search` and `/conversations/start`.

---

#### FR-004: Send messages (<=5000 chars) via WebSocket (P0)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit/Integration-in-memory):**
  - `src/backend/server.rs:738` (`test_websocket_upgrade_without_token`) – rejects unauthenticated upgrade.
  - `src/backend/server.rs:763` (`test_websocket_upgrade_with_invalid_token`) – rejects invalid token.
  - `src/backend/handlers/messages.rs:286` (`test_handle_message_to_online_recipient`) – processes inbound message envelope and acks.
  - `src/backend/handlers/dispatcher.rs:249` (`test_dispatcher_message_too_long`) – rejects oversize message envelope at WebSocket boundary.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/websocket_handshake_test.rs:116` (`T060-001`+) – handshake cases (requires wiring).
  - `tests/integration/message_delivery_test.rs:18` (`T096-001`+) – delivery pipeline tests (requires wiring).
- **Gaps:**
  - Full WebSocket client/server round-trip (tungstenite) not covered in executed suite.
- **Recommendation:** Add a server-spawned integration test (bind to ephemeral port) using `tokio-tungstenite` to validate end-to-end send/receive + length rejection.

---

#### FR-005: Real-time delivery or offline queue with indefinite retry + exponential backoff (P0)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit):**
  - `src/backend/services/message_queue.rs:326` (`test_queue_message`) – enqueues message for recipient.
  - `src/backend/services/message_queue.rs:349` (`test_load_pending_messages`) – loads pending messages from DB into in-memory queue.
  - `src/backend/services/message_queue.rs:339` (`test_exponential_backoff`) – validates retry schedule constants.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/message_delivery_test.rs:18` (`T096-001`+) – online delivery, offline queue + reconnect (requires wiring).
- **Gaps:**
  - No executed test proves retry loop + backoff scheduling against real time and state transitions.
  - “Indefinite retry until online or deleted” is not verified in executed suite.
- **Recommendation:** Add deterministic tests around retry scheduling/state transitions (consider injecting a time source instead of `chrono::Utc::now()` to enable `tokio::time::pause()`).

---

#### FR-006: Preserve and display complete message history (P1)

- **Coverage:** NONE ❌
- **Executed Tests:** None found verifying history retrieval correctness (ordering/pagination) at service or HTTP layer.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/message_delivery_test.rs:127` (`T096-003`) – pagination/order (requires wiring).
- **Recommendation:** Add executed tests for `MessageService::get_conversation_messages` and `/messages/history` (if applicable) including ordering + pagination.

---

#### FR-007: Show timestamps and sender information per message (P1)

- **Coverage:** NONE ❌
- **Executed Tests:** None found verifying message payload includes timestamp/sender info at API/WebSocket boundary.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/unit/models_test.rs:8` (`T502-003`) – message contains metadata (requires wiring).
- **Recommendation:** Add executed tests asserting delivered message payload includes sender id/username and timestamps.

---

#### FR-008: Presence online/offline updates within 1 second (P1)

- **Coverage:** NONE ❌
- **Executed Tests:** None found for `PresenceService` broadcast semantics or SLA.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/presence_test.rs:15` (`T105-001`+) – presence broadcast within 1s (requires wiring).
  - `tests/integration/presence_latency_test.rs:15` (`T106-001`) – latency SLA (requires wiring).
- **Recommendation:** Add executed tests for `PresenceService::mark_online/mark_offline` verifying broadcast recipients and latency.

---

#### FR-009: Search through message history by keywords (P1)

- **Coverage:** NONE ❌
- **Executed Tests:** None found validating `search_messages_in_conversation` behavior.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/search_test.rs:18` (`T097-001`+) – search cases (requires wiring).
- **Recommendation:** Add executed tests for `MessageService::search_messages_in_conversation` including authorization and empty result behavior.

---

#### FR-010: Logout and terminate session securely (P1)

- **Coverage:** NONE ❌
- **Executed Tests:** None found for `/auth/logout` handler or route behavior.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/logout_test.rs:8` (`T005-001`) – logout handler smoke test (requires wiring).
- **Gaps:**
  - JWT revocation/blacklist is not present (stateless JWT).
  - WebSocket disconnect on logout is not validated in executed suite.
- **Recommendation:** Add executed route tests ensuring logout requires auth and disconnects active sockets; document expected JWT behavior (stateless vs blacklist).

---

#### FR-011: Authorization checks on conversation queries; unauthorized => 403 (P0)

- **Coverage:** NONE ❌
- **Executed Tests:** None found verifying “non-participant cannot access conversation/messages” and receives 403 at HTTP boundary.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/e2e_test.rs:315` (`T100-004`) – intended authz checks but currently only tests self-chat prevention (requires correction + wiring).
- **Recommendation:** Add executed tests for `ConversationService::get_conversation_by_id` and `MessageService::get_conversation_messages` that ensure non-participants are rejected; add `warp::test` route tests asserting `403 Forbidden`.

---

#### FR-012: Store/protect user data (bcrypt, JWT 1h exp, SQLite MVP, disk encryption prod) (P0)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit):**
  - `src/backend/services/auth_service.rs:223` (`test_hash_password`) – bcrypt hash differs from password.
  - `src/backend/services/auth_service.rs:259` (`test_generate_token`) – token expiration is in the future.
- **Gaps:**
  - No executed test asserts JWT expiry is exactly 1 hour.
  - SQLite-at-rest encryption and production disk encryption are deployment concerns; no automated validation in repo.
- **Recommendation:** Add a unit test asserting `exp - iat == 3600` (within a small tolerance), and document operational controls for encryption scope (reference `T138_A`).

---

#### FR-013: Reject messages >5000 characters with user-facing validation error (P0)

- **Coverage:** FULL ✅
- **Executed Tests (Unit/WebSocket boundary):**
  - `src/backend/services/message_service.rs:360` (`test_validate_content_length`) – length validation rejects >5000.
  - `src/backend/handlers/dispatcher.rs:249` (`test_dispatcher_message_too_long`) – error response includes `INVALID_MESSAGE_LENGTH`.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/unit/message_validation_test.rs:54` (`T501-003`) – send_message rejects 5001 chars (requires wiring).
- **Recommendation:** Keep this behavior stable by treating `INVALID_MESSAGE_LENGTH` as part of the wire contract (document it in the protocol and avoid breaking changes).

---

#### FR-014: Support account deletion; cannot be reactivated (P0)

- **Coverage:** NONE ❌
- **Executed Tests:** None found for deletion handler or “cannot reactivate” constraint.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/deletion_test.rs:10` (`T004-001`+) – deletion scenarios (requires wiring).
- **Recommendation:** Add executed tests for `queries::soft_delete_user` + delete-account handler, and ensure login/signup behavior for deleted accounts is specified and tested.

---

#### FR-015: Anonymize messages from deleted users (P0)

- **Coverage:** NONE ❌
- **Executed Tests:** None found validating `anonymize_user_messages` behavior.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/integration/deletion_test.rs:61` (`T004-003`) – anonymization behavior (requires wiring).
- **Recommendation:** Add executed DB-level tests creating messages then running `soft_delete_user` and verifying `is_anonymized` is set; add API/UI contract expectations for “Deleted User”.

---

#### FR-016: Delivery status indicators (pending while queued; delivered once confirmed) (P1)

- **Coverage:** NONE ❌
- **Executed Tests:** None found validating status transitions/payloads across queue/delivery.
- **Supplemental (Not executed by `cargo test`):**
  - `tests/unit/models_test.rs:70` (`T502-005`) – message status helpers (requires wiring).
  - `tests/integration/message_delivery_test.rs:18` (`T096-001`+) – status updates (requires wiring).
- **Recommendation:** Add executed tests verifying status transitions (`pending -> queued/sent -> delivered`) and that frontend maps them to UI indicators.

---

#### FR-017: Rate-limit auth endpoints: 5 failed attempts/IP/15 minutes (P0)

- **Coverage:** PARTIAL ⚠️
- **Executed Tests (Unit/Route-level):**
  - `src/backend/middleware/rate_limit.rs:204` (`test_rate_limiter_blocks_after_max_attempts`) – limiter blocks after threshold.
  - `src/backend/server.rs:811` (`test_global_rate_limit_blocks_requests`) – global limiter returns 429.
  - `src/backend/server.rs:830` (`test_auth_rate_limit_blocks_after_failures`) is `#[ignore]` (auth limiter not validated via `warp::test`).
- **Gaps:**
  - No executed test validates auth rate limiting on `/auth/login` with the real rate limiter wiring and correct window (15 minutes).
- **Recommendation:** Refactor auth rate limiting to be testable via `warp::test` (inject IP + limiter dependency) and un-ignore the test; add a unit test asserting defaults are `5` attempts and `900s`.

---

### Gap Analysis

#### Critical Gaps (BLOCKER) ❌

P0 criteria without FULL coverage (blocks release/gate):

1. **FR-011 Authorization enforcement** – missing executed tests for 403 behavior.
2. **FR-014 Account deletion** – missing executed tests for delete + non-reactivation.
3. **FR-015 Message anonymization** – missing executed tests for anonymization correctness.
4. **FR-005 Offline retry/backoff** – no executed test for retry loop and scheduling.
5. **FR-017 Auth rate limiting** – endpoint-level behavior not executed (ignored test).

#### High Priority Gaps (PR BLOCKER) ⚠️

P1 criteria without FULL coverage:

- FR-006 message history, FR-007 timestamps/sender display, FR-008 presence SLA, FR-009 search, FR-010 logout, FR-016 delivery indicators.

---

### Tests with Issues

**BLOCKER Issues** ❌

- `tests/**` suite is not executed by `cargo test` (workspace wiring issue). Treat as non-actionable until moved into a crate `tests/` dir or added as a workspace member.

**WARNING Issues** ⚠️

- Several `tests/**` files exceed 300 lines and contain `sleep(...)` patterns per `docs/test-review.md` (these should be addressed when wiring the suite).

---

### Coverage by Test Level (Executed Automated Tests)

| Test Level | Criteria Covered | Notes |
| ---------- | ---------------- | ----- |
| E2E        | 0/17             | No executed E2E suite |
| API        | 0/17             | Limited `warp::test` coverage (health, websocket upgrade, headers) |
| Component  | 0/17             | No component test suite |
| Unit       | Partial          | Most executed coverage is unit-level within `src/backend/**` and `src/frontend/**` |

---

### Traceability Recommendations

**Immediate Actions (Before PR Merge)**

1. Wire `tests/**` into an executable test crate or migrate tests into crate-level integration tests.
2. Add executed P0 tests for authorization (403), deletion/anonymization, auth rate limiting, and offline retry scheduling.

**Short-term Actions (This Sprint)**

1. Add executed tests for message history, message search, presence updates, logout, and delivery status indicators.
2. Improve `warp::test` coverage for auth/signup/login paths with realistic IP handling.

---

## Integrated YAML Snippet (CI/CD)

```yaml
traceability_and_gate:
  traceability:
    story_id: "001-private-chat"
    date: "2025-12-17"
    coverage:
      overall: 6%
      p0: 10%
      p1: 0%
      p2: 0%
      p3: 0%
    gaps:
      critical: 9 # P0 criteria without FULL executed coverage
      high: 7 # P1 criteria without FULL executed coverage
      medium: 0
      low: 0
    quality:
      passing_tests: 149
      ignored_tests: 1
      failing_tests: 0
    evidence:
      test_results: "local:cargo test"
      traceability: "docs/traceability-matrix.md"
      gate_decision: "docs/gate-decision-story-001-private-chat.md"
      atdd: "docs/atdd-checklist-001-private-chat.md"
      automate: "docs/automation-summary.md"
```

## Related Artifacts

- `specs/001-private-chat/spec.md`
- `docs/gate-decision-story-001-private-chat.md`
- `docs/atdd-checklist-001-private-chat.md` (HALTED; framework missing)
- `docs/automation-summary.md` (HALTED; framework missing)
