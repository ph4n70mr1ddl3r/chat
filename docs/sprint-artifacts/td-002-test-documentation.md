# Story TD-002: Create Comprehensive Test Documentation

**Status:** 🆕 Backlog  
**Priority:** P2 (Medium - Test Quality)  
**Epic:** Technical Debt - Test Quality  
**Owner:** TBD  
**Created:** 2025-12-17

---

## 📋 Story

**As a** new contributor to the project  
**I want** comprehensive test documentation in `tests/README.md`  
**So that** I understand how to run tests, write new tests, and follow established patterns

---

## 🎯 Acceptance Criteria

### AC1: Setup Instructions ✓
- Document how to run all test types: `cargo test`, `cargo test --test integration`
- Document test environment setup (no special setup needed for this project)
- Document how to run specific tests: `cargo test test_name`
- **Test:** New developer can run tests following README

### AC2: Architecture Overview ✓
- Document test organization (integration/, unit/, contract/, load/, fixtures/, helpers/)
- Explain fixture pattern (`setup_test_db()`, `create_users_and_conversation()`)
- Explain data factory pattern (`create_test_user()` with UUIDs)
- Explain deterministic polling (`poll_until()` helper)
- **Test:** README explains all test patterns clearly

### AC3: Best Practices ✓
- Document test isolation (in-memory DB per test)
- Document deterministic waits (use `poll_until()`, avoid `sleep()`)
- Document test IDs (T100-001, T060-001 format)
- Document BDD structure (Given-When-Then comments)
- **Test:** README provides clear guidance on writing quality tests

### AC4: Running Tests in CI ✓
- Document how tests run in CI/CD (future: when CI is set up)
- Document test artifacts (test-results/, coverage/)
- Document performance targets (tests should complete in \u003c2 minutes)
- **Test:** CI integration documented

---

## 🔨 Tasks & Subtasks

### Task 1: Create tests/README.md Structure
- [ ] Create `/tests/README.md` file
- [ ] Add table of contents
- [ ] Add overview section

### Task 2: Document Test Organization
- [ ] Document directory structure (integration/, unit/, contract/, load/)
- [ ] Document fixture architecture
- [ ] Document helper patterns
- [ ] Include code examples

### Task 3: Document Running Tests
- [ ] Add "Running Tests" section
- [ ] Document `cargo test` commands
- [ ] Document test filtering
- [ ] Document test output interpretation

### Task 4: Document Best Practices
- [ ] Add "Writing Tests" section
- [ ] Document fixture usage
- [ ] Document factory pattern
- [ ] Document `poll_until()` usage
- [ ] Include anti-patterns (what NOT to do)

### Task 5: Document Test Patterns
- [ ] Highlight excellent patterns from existing tests
- [ ] Reference `poll_until()` from helpers/polling.rs
- [ ] Reference factory pattern from helpers/factories.rs
- [ ] Include example test with annotations

---

## 📊 Definition of Done Checklist

- [ ] **AC1:** Setup instructions complete
- [ ] **AC2:** Architecture overview complete  
- [ ] **AC3:** Best practices documented
- [ ] **AC4:** CI integration documented
- [ ] **README Created:** `/tests/README.md` exists
- [ ] **Examples Included:** Code examples for key patterns
- [ ] **Links Working:** All file references link correctly
- [ ] **Review:** README reviewed for clarity

---

## 📈 Estimation

**Size:** S (1-2 hours)  
**Complexity:** Low (documentation task)  
**Risk:** Low (no code changes)  
**Time Breakdown:**
- README structure: 15 minutes
- Test organization section: 20 minutes
- Running tests section: 15 minutes
- Best practices section: 30 minutes
- Examples and polish: 20 minutes
- **Total: 1.5 hours**

---

## 📁 File References

### Files to Create
- `tests/README.md` (new file)

### Reference Files
- `tests/helpers/polling.rs` - Document `poll_until()` pattern
- `tests/helpers/factories.rs` - Document factory pattern
- `tests/fixtures/database.rs` - Document fixture pattern
- `tests/integration/e2e_test.rs` - Example test with BDD structure
- `test-quality-review.md` - Source of best practices

---

## 📝 README.md Template

```markdown
# Test Suite Documentation

## Overview
Comprehensive test suite for the Rust chat application...

## Test Organization
- `integration/` - Full workflow tests
- `unit/` - Unit tests (colocated with source)
- `contract/` - API schema validation
- `load/` - Performance testing
- `fixtures/` - Shared test data setup
- `helpers/` - Test utilities

## Running Tests
\`\`\`bash
# Run all tests
cargo test

# Run specific test type
cargo test --test integration

# Run specific test
cargo test test_complete_user_flow
\`\`\`

## Best Practices
### 1. Use Deterministic Polling
❌ **Don't:** `tokio::time::sleep(Duration::from_millis(100)).await`
✅ **Do:** `poll_until(Duration::from_secs(2), || async { ... }).await`

### 2. Use Data Factories
❌ **Don't:** Hardcode test data
✅ **Do:** Use `create_test_user(Default::default())`

## Writing Tests
[Examples and guidance...]
```

---

## 🏷️ Labels & Metadata

- **Epic:** Technical Debt - Test Quality
- **Type:** Documentation
- **Priority:** P2 (Medium)
- **Complexity:** Low
- **Risk:** Low
- **Tech Stack:** Markdown
- **Story Points:** 2 (S = 1-2 hours)
