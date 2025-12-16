# Definition of Ready (DoR) & Definition of Done (DoD)

**Version:** 1.0  
**Date:** December 16, 2025  
**Audience:** All developers, designers, testers, product managers  
**Purpose:** Establish consistent entry/exit criteria for all 21 sprint stories to prevent scope creep, surprises, and mid-sprint negotiations.

---

## Overview

Every story must pass DoR before implementation starts, and DoD before code merges to main.

**DoR (Definition of Ready):** Story is clear, complete, and ready for developer to code  
**DoD (Definition of Done):** Story implementation is complete, tested, reviewed, and ready for production

---

## DEFINITION OF READY (DoR)

**Use this checklist BEFORE a story enters the sprint.**

A story is ready when ALL items below are ✓:

### DoR Checklist

#### 1. Story Title & Description ✓

- [ ] Title is specific and actionable (not vague)
  - ❌ WRONG: "Build stuff"
  - ✅ RIGHT: "Implement Button component with 4 variants"
  
- [ ] Description explains WHAT and WHY, not HOW
  - ❌ WRONG: "Create a button by making a rectangle and adding text"
  - ✅ RIGHT: "Users need a consistent button for all actions. Button must support 4 variants (primary, secondary, tertiary, danger) with proper keyboard accessibility."

- [ ] Story type is clear: Feature / Bug / Technical Debt / Documentation

---

#### 2. Acceptance Criteria (AC) are TESTABLE ✓

- [ ] 5+ acceptance criteria defined
- [ ] Each AC is specific and measurable (not vague)
  - ❌ WRONG: "Button should be styled nicely"
  - ✅ RIGHT: "Button primary variant renders with Fluent Blue (#0078D4) background, white text, 36px height, 14px font"

- [ ] Each AC can be verified with a test
- [ ] Each AC includes an example or screenshot
- [ ] AC link to UX spec section for design reference

**Example AC for Button story:**
```
AC 1: Button renders with correct label
- Given: Button component with label="Send"
- When: Component renders
- Then: Text "Send" is visible in button

AC 2: on_clicked callback fires when clicked
- Given: Button with on_clicked callback
- When: User clicks button
- Then: Callback fires with no arguments

AC 3: Keyboard accessible (Enter/Space activation)
- Given: Button with focus
- When: User presses Enter or Space
- Then: Button activates (same as click)

AC 4: Respects reduce_motion preference
- Given: Button with is_loading=true and reduce_motion=true
- When: Component renders
- Then: Spinner is static (no rotation animation)

AC 5: All 4 variants render correctly
- When: Rendering all variants (primary, secondary, tertiary, danger)
- Then: Colors, borders, and text colors match UX spec
```

---

#### 3. AC Reference UX Spec ✓

- [ ] Every AC links to UX spec section where design is defined
- [ ] Design rationale is explained (not just "because design said so")

**Example:**
```
AC 1: Button primary variant uses Fluent Blue
Reference: UX Spec Section 7.2 (Visual System - Colors)
Rationale: Primary blue (#0078D4) creates visual hierarchy and draws user attention to main actions
```

---

#### 4. Dependencies are EXPLICIT ✓

- [ ] Story lists what it BLOCKS (dependent stories can't start until this is done)
- [ ] Story lists what BLOCKS IT (stories that must be done first)
- [ ] All blocking stories are either DONE or on sprint board

**Example:**
```
BLOCKS: US-002, US-003, US-004, US-005, US-006
  (Button, TextField, Chip, Icon, LoadingSpinner all use design tokens)

BLOCKED BY: None
  (Design tokens are first story, no dependencies)

RELATED: None
```

**Rule:** If story is blocked by something not done: Story stays in Backlog. Not ready.

---

#### 5. Story Size Fits ONE SPRINT WEEK ✓

- [ ] Story points: 5-8 points (fits 1 week, ~25-35 hours for developer)
- [ ] If > 8 points: Story is too big. Split into smaller stories.
- [ ] Complexity: low/medium/high assessed
- [ ] Risk: low/medium/high assessed

**Example:**
```
Story Points: 5
Complexity: Medium (component API is clear, but accessibility needs careful testing)
Risk: Low (uses established component pattern, no external dependencies)
Time estimate: 25-30 hours (design tokens 4h + implementation 15h + testing 6h + review 2h)
```

---

#### 6. Non-Functional Requirements are EXPLICIT ✓

- [ ] Performance targets defined (if applicable)
- [ ] Accessibility targets defined (WCAG AA minimum)
- [ ] Motion/animation preferences defined
- [ ] Responsive/responsive breakpoints defined (if applicable)

**Example:**
```
Performance: Component must render in < 16ms (60 FPS)
Accessibility: WCAG 2.1 Level AA minimum
  - Keyboard navigable (Tab, Enter, Space)
  - Screen reader compatible (role + label)
  - 4.5:1 color contrast
Motion: Must respect reduce_motion preference
  - When reduce_motion=true: no animations
  - Static icon instead of spinner
Responsive: Works at 320px (mobile) to 1920px (desktop)
```

---

#### 7. Test Plan is ATTACHED ✓

- [ ] Unit test cases defined (what to test in isolation)
- [ ] Integration test cases defined (how component works with parents)
- [ ] Accessibility test cases defined (screen reader, keyboard)
- [ ] Visual/edge case test cases defined

**Example:**
```
Unit Tests:
- [ ] Button renders with correct label for all sizes
- [ ] Button renders with correct color for all variants
- [ ] on_clicked fires when clicked
- [ ] on_clicked fires when Enter pressed
- [ ] on_clicked fires when Space pressed
- [ ] is_disabled prevents clicks
- [ ] is_loading shows spinner and hides label
- [ ] Spinner is static when reduce_motion=true

Integration Tests:
- [ ] Button works inside MessageInput container
- [ ] Button click updates parent state
- [ ] Multiple buttons in form are independently clickable

Accessibility Tests:
- [ ] NVDA announces "Button: [label]"
- [ ] Tab key navigates to button
- [ ] Enter activates button
- [ ] Space activates button
- [ ] Focus ring is visible
- [ ] Color contrast is 4.5:1

Visual Tests:
- [ ] Primary variant matches UX spec
- [ ] Secondary variant matches UX spec
- [ ] Tertiary variant matches UX spec
- [ ] Danger variant matches UX spec
- [ ] Hover states match spec
- [ ] Focus states match spec
- [ ] Disabled state matches spec
```

---

#### 8. Owner & Backup Assigned ✓

- [ ] Primary developer assigned (name + GitHub)
- [ ] Backup developer identified (if primary is blocked)
- [ ] Designer assigned for design review (usually Sally)
- [ ] Reviewer assigned for code review (usually Winston)

**Example:**
```
Owner: Amelia (Developer)
Backup: Barry (Quick Flow Dev)
Designer Review: Sally
Code Review: Winston
Test Review: Murat
```

---

#### 9. Any Known Risks/Blockers Documented ✓

- [ ] Technical risks identified ("This uses WebSocket for first time")
- [ ] Design risks identified ("Need clarification on hover behavior")
- [ ] Dependencies on external systems listed
- [ ] Mitigation strategies proposed

**Example:**
```
Risk: TextField is first text input component, pattern not established yet
Mitigation: Reference React best practices + Slint documentation

Risk: Accessibility testing requires screen reader (NVDA), team may not be familiar
Mitigation: Murat will pair with developer for first A11y test

Risk: Design tokens may change mid-week, requiring component rework
Mitigation: Lock tokens by Monday EOD before component work starts
```

---

#### 10. Design Approved ✓

- [ ] Component definition document reviewed (WEEK1_COMPONENT_DEFINITIONS.md)
- [ ] UX spec references reviewed
- [ ] Variants/sizes match design system
- [ ] Sally (UX Designer) has approved story

**Sign-off:** Sally has reviewed and approved ✓

---

### DoR Process

**Step 1: Product creates story** (Bob + John)
- Title, description, AC

**Step 2: Designer reviews** (Sally)
- Verifies AC match UX spec
- Approves design references
- Updates AC if needed

**Step 3: Architect reviews** (Winston)
- Identifies dependencies
- Flags technical risks
- Ensures size is reasonable

**Step 4: Scrum Master finalizes** (Bob)
- Completes DoR checklist
- Assigns owner/backup
- Moves to "Ready" column on sprint board

**Step 5: Developer can start**
- Story is in "Ready" column
- All information is available
- No guessing needed

**If ANY box unchecked:** Story goes back to Backlog. Cannot start.

---

## DEFINITION OF DONE (DoD)

**Use this checklist BEFORE code merges to main.**

A story is done when ALL items below are ✓:

### DoD Checklist

#### 1. All Acceptance Criteria PASS ✓

- [ ] Every single AC verified with proof
- [ ] Test output or screenshot proving AC is met
- [ ] Developer ran test locally and confirmed pass

**Example:**
```
AC 1: Button renders with label ✓ (unit test: test_button_renders_with_label)
AC 2: on_clicked fires ✓ (unit test: test_on_clicked_fires_when_clicked)
AC 3: Keyboard accessible ✓ (unit test: test_keyboard_enter_activates_button)
AC 4: reduce_motion works ✓ (unit test: test_reduce_motion_disables_animation)
AC 5: Variants correct ✓ (visual test: screenshot comparison)
```

**Rule:** If ANY AC fails, story is NOT done. Reopen and fix.

---

#### 2. Unit Tests 100% PASSING ✓

- [ ] All tests pass locally: `cargo test --all`
- [ ] No skipped tests (`#[ignore]`)
- [ ] No flaky tests (don't pass sometimes, fail sometimes)
- [ ] Test coverage: every AC has at least 1 test

**Output must show:**
```
running 47 tests
test component_tests::test_button_renders_with_label ... ok
test component_tests::test_on_clicked_fires ... ok
test component_tests::test_keyboard_accessible ... ok
...
result: ok. 47 passed; 0 failed; 0 skipped
```

**Rule:** 100% passing or story does not merge. Zero exceptions.

---

#### 3. Integration Tests PASSING ✓

- [ ] Component tested with parent component (not in isolation)
- [ ] Component tested with real state changes (not mocks only)
- [ ] Component tested with real callbacks (not no-op)

**Example for Button:**
```
Integration test: Button in MessageInput
- [ ] MessageInput renders Button
- [ ] Click Button triggers MessageInput's on_send callback
- [ ] MessageInput state updates correctly

Integration test: Button in ConversationHeader
- [ ] ConversationHeader renders 2 buttons
- [ ] Each button click triggers correct action
- [ ] Buttons don't interfere with each other
```

---

#### 4. Accessibility Tests PASSING ✓

For EVERY component, verify:

- [ ] **Screen Reader:** Component announced correctly
  - Tool: NVDA (Windows) or VoiceOver (Mac)
  - Verify: Role, label, and interactive elements announced
  - Example: Button announces "Button: Send Message"

- [ ] **Keyboard Navigation:** All interactions keyboard accessible
  - Tab key navigates to component
  - Enter / Space activates primary action
  - Arrow keys navigate in lists (if applicable)
  - Focus ring visible

- [ ] **Focus Management:** Focus ring visible and clear
  - Ring color: high contrast (at least 3:1)
  - Ring visible on keyboard nav
  - Ring removed on mouse click (if applicable)

- [ ] **Color Contrast:** Text readable by color-blind users
  - Text on background: 4.5:1 (WCAG AA)
  - UI components: 3:1 (WCAG AA)
  - Test tool: WebAIM contrast checker

- [ ] **Motion Preferences:** Animations respect reduce_motion
  - If reduce_motion=true: no animations, static icons
  - If reduce_motion=false: animations enabled
  - Test: screenshot comparison reduce_motion=true vs false

**Output must show:**
```
Accessibility Test Results for Button
✓ NVDA announces "Button: Send Message"
✓ Tab navigation works
✓ Enter activates button
✓ Space activates button
✓ Focus ring visible (color #0078D4, 2px)
✓ Text contrast 4.5:1 (pass WCAG AA)
✓ reduce_motion=true: spinner is static
✓ reduce_motion=false: spinner rotates
```

---

#### 5. Code Review APPROVED ✓

**Reviewer: Winston (Architect)**

Reviewer checks:
- [ ] Code follows Rust 1.75+ conventions
- [ ] Component follows COMPONENT_API_STANDARD.md
- [ ] Props organized (Data / Behavior / Style)
- [ ] Props limit: ≤12 total
- [ ] No two-way binding (in-out properties)
- [ ] Callbacks properly named (on_<subject>_<verb>)
- [ ] No accessibility violations
- [ ] No performance issues
- [ ] Tests comprehensively cover AC + edge cases
- [ ] Documentation clear and complete
- [ ] No dead code, debug prints, or warnings
- [ ] Follows project architecture (state management pattern)

**Approval required. If reviewer says "request changes":**
- Developer receives feedback
- Developer addresses issues
- Developer re-requests review
- Merge blocked until approved

---

#### 6. Design Compliance VERIFIED ✓

**Reviewer: Sally (UX Designer)**

Sally verifies component matches UX spec exactly:

- [ ] Visual design matches spec
  - Colors correct (use design tokens, not hardcoded)
  - Typography correct (font sizes, weights, spacing)
  - Spacing matches spec (padding, margins, gaps)
  - Sizing correct (component dimensions)

- [ ] Interactions match spec
  - Hover states match spec
  - Focus states match spec
  - Active/pressed states match spec
  - Disabled states match spec

- [ ] Animations match spec (if applicable)
  - Duration correct (e.g., 400ms)
  - Easing correct (linear, ease-out, etc.)
  - Curve smooth and natural
  - Respects reduce_motion

- [ ] Variants all present
  - Primary variant present and correct
  - Secondary variant present and correct
  - All size variants present and correct

**Output must show:**
```
Design Compliance Review for Button
✓ Primary variant: Fluent Blue (#0078D4) matches spec
✓ Secondary variant: outline style matches spec
✓ Hover state: darker blue matches spec
✓ Focus state: 2px blue outline matches spec
✓ Disabled state: light gray matches spec
✓ All 3 sizes present: small (28px), medium (36px), large (44px)
✓ Uses design tokens (no hardcoded colors)
✓ Typography matches spec
✓ Spacing matches spec
```

**Design compliance is NOT optional. Fix or don't ship.**

---

#### 7. Documentation COMPLETE ✓

- [ ] **Component Definition Document** written
  - File: `/docs/components/ButtonComponentDefinition.md`
  - Includes: purpose, props, callbacks, variants, accessibility, testing, examples
  - Reference: WEEK1_COMPONENT_DEFINITIONS.md template

- [ ] **Code Comments** explain complex logic
  - Comments on non-obvious Slint syntax
  - Comments on state management patterns
  - Comments on accessibility workarounds

- [ ] **Examples** provided in code
  - Usage examples in story or README
  - Real-world usage scenarios shown

- [ ] **Known Limitations** documented
  - What this component doesn't do
  - Why certain design choices were made
  - Known edge cases

- [ ] **README** updated (if applicable)
  - Component listed in component inventory
  - Links to component docs

**Example Documentation:**
```markdown
# Button Component

## Purpose
Primary interactive element for all actions. Available in 4 variants and 3 sizes.

## Props
[Full props table from COMPONENT_API_STANDARD]

## Variants
- primary (default)
- secondary
- tertiary
- danger

## Accessibility
- Role: button
- Label: button text
- Keyboard: Tab, Enter, Space
- Motion: respects reduce_motion

## Examples
[Usage code]

## Known Limitations
- No icon support (use Icon separately)
- Fixed button width (set by label length)
```

---

#### 8. Performance VERIFIED ✓

- [ ] **Render Performance:** Component renders in < 16ms (60 FPS)
  - Measured with profiler: `cargo profile` or Slint debugger
  - Result: < 16ms per frame

- [ ] **Memory Usage:** No memory leaks
  - Component destroyed properly when unmounted
  - No dangling references

- [ ] **Target Performance Met:**
  - Story specified performance target
  - Actual performance meets or exceeds target
  - Results documented

**Example:**
```
Performance Test for Button
- Render 1000 buttons: 12ms (target: < 16ms) ✓
- Memory per instance: 2.5 KB (no leaks) ✓
- Animation (spinner) 60 FPS (target: 60 FPS) ✓
```

---

#### 9. No Warnings or Clippy Issues ✓

```bash
$ cargo clippy --all-targets
Checking private_chat_frontend v0.1.0
Finished dev [unoptimized + debuginfo] target(s) in 0.42s
```

**Rule:** Zero warnings or story cannot merge.

- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] No unused variables
- [ ] No debug prints left in code

**If warnings exist:**
- Developer must fix or justify exception
- Exception must be documented with `#[allow(...)]`
- Exception requires code reviewer approval

---

#### 10. Merged to Main Branch ✓

Only after ALL above items are ✓:

- [ ] PR created with clear description
- [ ] CI passes (tests + clippy)
- [ ] Code review approved (Winston)
- [ ] Design review approved (Sally)
- [ ] Accessibility review approved (Murat)
- [ ] PR merged to main branch
- [ ] Story moved to "Done" column on sprint board

**Output:**
```
PR #42 merged to main
Commit: a1b2c3d "Implement Button component (US-002)"
Tests: 47 passed, 0 failed
Clippy: 0 warnings
Reviews: 3 approvals (Winston, Sally, Murat)
```

---

### DoD Process

**Step 1: Developer implements locally**
- Code passes `cargo test --all` locally
- Code passes `cargo clippy --all-targets` locally

**Step 2: Developer pushes and creates PR**
- CI runs automatically
- Tests must pass (or PR is blocked)
- Clippy check must pass (or PR is blocked)

**Step 3: Code Review (Winston)**
- Winston reviews against API Standard + architecture
- Requests changes if needed
- Approves when satisfied

**Step 4: Design Review (Sally)**
- Sally checks visual design against spec
- Requests changes if needed
- Approves when satisfied

**Step 5: Accessibility Review (Murat)**
- Murat checks keyboard, screen reader, motion, contrast
- Requests changes if needed
- Approves when satisfied

**Step 6: Developer addresses feedback**
- If no changes needed: goes to step 7
- If changes needed: developer fixes, re-requests reviews

**Step 7: Merge to main**
- All approvals in place
- All tests passing
- Merge button clicked
- Story automatically moves to "Done"

**If ANY item fails:** Story goes back to In Progress. Create bug/issue to track.

---

## SPECIAL RULES

### Rule 1: Blocked Stories

If a story is blocked mid-sprint (e.g., waiting for something):

- [ ] File issue: "US-010 blocked by X"
- [ ] Story is paused (not abandoned)
- [ ] Developer moves to another story
- [ ] When blocker resolves: story resumes
- [ ] Story deadline extends by blocker duration

**Example:**
```
Story: US-010 (Message Delivery)
Blocked by: US-001 (Design Tokens) merge not yet approved

Action:
- Pause US-010
- Developer pulls US-011 (another story)
- Once US-001 merges: resume US-010
```

### Rule 2: AC Changes Mid-Sprint

If AC needs to change during implementation:

- [ ] File issue: "AC for US-003 needs clarification"
- [ ] Do NOT change AC in sprint without approval
- [ ] Product (Bob) + Designer (Sally) review change
- [ ] If impact < 1 hour: approve + continue
- [ ] If impact > 1 hour: defer to next sprint

**Example:**
```
Developer realizes: "AC says button should spin, but never says HOW FAST"
Action: File issue, discuss with Sally
Decision: Sally specifies "400ms rotation" (< 1 hour impact)
Result: AC updated, developer continues with 30 min adjustment
```

### Rule 3: Story Finished Early

If developer finishes a story before EOW:

- [ ] All DoD items complete ✓
- [ ] Story merged to main ✓
- [ ] Developer pulls next story from sprint backlog
- [ ] If no sprint stories left: pulls from next sprint
- [ ] Goal: keep velocity high, minimize idle time

**Example:**
```
Developer finishes US-002 (Button) on Wednesday
Next story: US-003 (TextField)
Pull from sprint, continue coding
Extra day: TextField finishes early too
Result: 2 stories done by Thursday (velocity bonus)
```

### Rule 4: Story Overruns

If story is not done by EOW:

- [ ] Assessment: What's left?
- [ ] If < 2 hours work: carry over to next sprint
- [ ] If > 2 hours work: split story for next sprint
- [ ] Document why overrun occurred
- [ ] Adjust future estimates based on learning

**Example:**
```
Story US-005 (Chip) estimated 5 points
Actual: 7 points (accessibility testing took longer)

Reflection:
- Accessibility testing underestimated
- Team is new to NVDA testing
- Future stories: add 1-2 hours for A11y testing

Action:
- Carry remaining 2 hours to next sprint
- Adjust future estimates +1-2 hours
```

---

## ENFORCEMENT

### Automated Enforcement (CI/CD)

These items are automatically enforced by CI:
- Unit tests 100% passing (or PR blocks)
- Clippy zero warnings (or PR blocks)
- All AC have test coverage (or reviewer catches)

### Manual Enforcement (Code Review)

These items are manually verified:
- AC matches UX spec (Sally)
- Code follows standards (Winston)
- Accessibility verified (Murat)
- Performance targets met (Murat)

### Process Enforcement (Sprint Board)

These items are process-enforced:
- DoR checked before story enters sprint (Bob)
- DoD checked before story merges (Bob)
- Only stories in "Ready" column can be pulled
- Story cannot merge without both design + code review

---

## VELOCITY TRACKING

Bob tracks velocity to predict shipping dates:

```
Week 1 Sprint:
- Planned: 30 points (6 stories)
- Completed: 25 points (5 stories)
- Velocity: 25 points / week

Reason for 5-point miss:
- US-005 (Chip) took longer due to A11y testing
- Adjusted future estimates: +1 hour for accessibility

Week 2 Projection:
- 21 points remaining stories
- Velocity: 25 points / week
- ETA: 0.8 weeks (by Wednesday of Week 2)
- Confidence: Medium (new pattern, may adjust)
```

**Velocity helps us:**
- ✅ Predict which features ship when
- ✅ Adjust team capacity if needed
- ✅ Identify bottlenecks early
- ✅ Improve estimation accuracy over time

---

## CHECKLIST: READY BEFORE WEEK 1

- [ ] All 21 stories have DoR completed ✓
- [ ] All DoD items understood by team
- [ ] CI/CD configured to enforce DoD items
- [ ] Sprint board set up with columns (Backlog, Ready, In Progress, In Review, Done)
- [ ] Developers know how to run tests locally
- [ ] Reviewers (Winston, Sally, Murat) briefed on DoD items
- [ ] Accessibility tools (NVDA, contrast checker) installed
- [ ] Velocity tracking spreadsheet set up

---

**Approved By:** Bob (Scrum Master), Winston (Architect), Sally (UX Designer), Amelia (Developer)  
**Effective Date:** Week 1 (Dec 18-22, 2025)  
**Reviewed Every:** Weekly (adjust based on actual velocity + team feedback)  
**Next Review:** Dec 23, 2025 (end of Week 1)

