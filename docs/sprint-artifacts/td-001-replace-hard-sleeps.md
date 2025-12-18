# Story TD-001: Replace Hard Sleeps with Deterministic Polling

**Status:** 🆕 Backlog  
**Priority:** P2 (Medium - Test Quality)  
**Epic:** Technical Debt - Test Quality  
**Owner:** TBD  
**Created:** 2025-12-17

---

## 📋 Story

**As a** test engineer  
**I want** to replace hardcoded sleep calls with deterministic polling  
**So that** tests are faster, more reliable, and don't have timing-dependent flakiness

---

## 🎯 Acceptance Criteria

### AC1: Replace Sleep in e2e_test.rs Line 209 ✓
- **Current:** `tokio::time::sleep(Duration::from_millis(50 * login_attempt as u64)).await;`
- **Replace with:** `poll_until()` helper waiting for login success
- **Benefit:** Test completes as soon as login succeeds, not fixed delay
- **Test:** E2E test still passes, completes faster on average

### AC2: Replace Sleep in e2e_test.rs Line 151 ✓  
- **Current:** `tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;`
- **Replace with:** Poll for message persistence before sending next message
- **Benefit:** Deterministic ordering without arbitrary delays
- **Test:** Message ordering test still passes

### AC3: Replace Sleep in message_delivery_test.rs Line 151 ✓
- **Current:** `tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;`
- **Replace with:** Poll for database persistence
- **Benefit:** Test completes as soon as message persists
- **Test:** Message delivery test still passes

---

## 🔨 Tasks & Subtasks

### Task 1: Replace e2e_test.rs Login Retry Sleep
- [ ] Update line 209 to use `poll_until()` for login success
- [ ] Remove exponential backoff sleep, use polling timeout instead
- [ ] Test: Verify login test still passes

### Task 2: Replace e2e_test.rs Message Ordering Sleep  
- [ ] Update line 151 to poll for message persistence
- [ ] Use `db::queries::find_message_by_id()` in poll condition
- [ ] Test: Verify message ordering is preserved

### Task 3: Replace message_delivery_test.rs Persistence Sleep
- [ ] Update line 151 to poll for message in database
- [ ] Use 500ms max timeout (faster than 10ms * many messages)
- [ ] Test: Verify all delivery tests pass

### Task 4: Verify All Tests Pass
- [ ] Run `cargo test` - all tests passing
- [ ] Check test execution time - should be same or faster
- [ ] No regressions introduced

---

## 📊 Definition of Done Checklist

- [ ] **AC1:** e2e_test.rs:209 uses `poll_until()` instead of sleep
- [ ] **AC2:** e2e_test.rs:151 polls for persistence
- [ ] **AC3:** message_delivery_test.rs:151 polls for persistence
- [ ] **All Tests Pass:** `cargo test` shows 100% pass rate
- [ ] **No Regressions:** Test execution time same or better
- [ ] **Code Review:** Changes reviewed and approved

---

## 📈 Estimation

**Size:** XS (30 minutes - 1 hour)  
**Complexity:** Low (pattern already exists in helpers/polling.rs)  
**Risk:** Low (excellent existing `poll_until()` helper)  
**Time Breakdown:**
- Replace 3 sleep calls: 20 minutes
- Test verification: 10 minutes
- **Total: 30 minutes**

---

## 📁 File References

### Files to Modify
- `tests/integration/e2e_test.rs` (lines 151, 209)
- `tests/integration/message_delivery_test.rs` (line 151)

### Reference Files
- `tests/helpers/polling.rs` - Use `poll_until()` helper
- `test-quality-review.md` - Original issue identification

---

## 🏷️ Labels & Metadata

- **Epic:** Technical Debt - Test Quality
- **Type:** Test Improvement
- **Priority:** P2 (Medium)
- **Complexity:** Low
- **Risk:** Low
- **Tech Stack:** Rust, Tokio, `poll_until()` helper
- **Story Points:** 1 (XS = 30min-1h)
