# Test Automation Expansion (Automate) — HALTED

**Workflow:** `testarch-automate` (`*automate`)
**Date:** 2025-12-17
**Evaluator:** TEA (Murat)

## Status

**HALTED (framework missing)**

Per the workflow’s preflight requirements, automated generation requires a configured E2E test harness (`playwright.config.*` or `cypress.config.*`) plus a Node dependency manifest (`package.json`) to install and run it.

## Preflight Evidence

- `package.json`: **missing** at repo root
- `playwright.config.*` / `cypress.config.*` / `cypress.json`: **not found**
- Playwright/Cypress deps in repo: **not detected**

## Why This Matters for This Repo

This repository is Rust-native (Rust + Slint + Warp/Tokio). The BMAD `*automate` workflow is explicitly oriented around Playwright/Cypress scaffolds, and will not proceed without that harness.

## Options to Unblock

1. **Adopt a Node-based E2E harness**
   - Add `package.json`
   - Run `*framework` to scaffold Playwright/Cypress
   - Re-run `*automate`

2. **Stay Rust-native (recommended here)**
   - Make `tests/**` executable by moving key tests into a crate `tests/` directory (e.g. `src/backend/tests/…`) or by adding a dedicated workspace member crate that compiles them.
   - Add/enable missing P0 coverage identified in `docs/traceability-matrix.md` and re-run `cargo test` + `*trace`.

## Related Evidence

- Traceability and gate decision: `docs/traceability-matrix.md`, `docs/gate-decision-story-001-private-chat.md`
- Test suite wiring note: `docs/test-review.md`

