# Authenticated Provider Visibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Hide Claude or Codex from the Mana roster only when that provider has no usable local authentication credentials.

**Architecture:** Replace the poller's ambiguous optional fetch result with an enum that distinguishes missing credentials from authenticated request failures and successful usage. Serialize that distinction as `authenticated` on each snapshot, then let a pure frontend helper control the provider section's `hidden` attribute while preserving compatibility with payloads that predate the field.

**Tech Stack:** Rust, Tauri 2, serde, TypeScript, Vitest

## Global Constraints

- Missing or malformed local credentials hide that provider.
- Authenticated providers remain visible through temporary API or parsing failures.
- Removing credentials after a successful fetch hides the provider on the next poll.
- Missing `authenticated` in a frontend payload defaults to visible.
- Widget height is recalculated after provider visibility changes.

---

### Task 1: Preserve Authentication State in Polling

**Files:**
- Modify: `src-tauri/src/poll.rs`

**Interfaces:**
- Produces: `UsageSnapshot.authenticated: bool`
- Produces: `FetchResult::{Unauthenticated, Failed, Success { bars, plan }}`
- Consumes: existing `creds::read_claude_creds()` and `creds::read_codex_creds()` credential readers

- [ ] **Step 1: Write failing state-transition tests**

Add assertions that `Unauthenticated` creates an unauthenticated absent snapshot, `Failed` without history creates an authenticated absent snapshot, `Failed` with history preserves authenticated stale data, credential removal discards history, and a later successful result restores an authenticated fresh snapshot.

```rust
#[test]
fn missing_credentials_are_unauthenticated() {
    let s = fold_snapshot(None, "claude", FetchResult::Unauthenticated, 100);
    assert!(!s.authenticated);
    assert_eq!(s.status, "absent");
}

#[test]
fn authenticated_failure_without_history_remains_visible() {
    let s = fold_snapshot(None, "codex", FetchResult::Failed, 100);
    assert!(s.authenticated);
    assert_eq!(s.status, "absent");
}
```

- [ ] **Step 2: Run the Rust tests and verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml poll::tests`

Expected: compilation fails because `FetchResult` has no enum variants and `UsageSnapshot` has no `authenticated` field.

- [ ] **Step 3: Implement the explicit fetch result**

Define the result and snapshot field, then make each fetch function read credentials first:

```rust
pub enum FetchResult {
    Unauthenticated,
    Failed,
    Success { bars: Vec<Bar>, plan: Option<String> },
}
```

Return `Unauthenticated` only when the provider's credential reader returns `None`; return `Failed` for request, HTTP, JSON, or parsing failures. Update `fold_snapshot` so only `Failed` retains history as stale, while `Unauthenticated` creates a fresh hidden snapshot with empty bars.

- [ ] **Step 4: Run the Rust tests and verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml poll::tests`

Expected: all poller state-transition tests pass.

- [ ] **Step 5: Commit the backend change**

```bash
git add src-tauri/src/poll.rs
git commit -m "feat: distinguish provider authentication state"
```

---

### Task 2: Hide Unauthenticated Provider Sections

**Files:**
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: optional `Snapshot.authenticated`
- Produces: `providerIsVisible(snapshot: Snapshot | undefined): boolean`

- [ ] **Step 1: Write failing frontend visibility tests**

Extend the snapshot fixture and add focused assertions:

```ts
it("hides only explicitly unauthenticated providers", () => {
  expect(providerIsVisible({ ...weeklyOnly, authenticated: false })).toBe(false);
  expect(providerIsVisible({ ...weeklyOnly, authenticated: true, status: "absent", bars: [] })).toBe(true);
});

it("keeps startup and legacy snapshots visible", () => {
  expect(providerIsVisible(undefined)).toBe(true);
  expect(providerIsVisible(weeklyOnly)).toBe(true);
});
```

- [ ] **Step 2: Run the frontend test and verify RED**

Run: `npm test -- src/view.test.ts`

Expected: compilation fails because `providerIsVisible` is not exported.

- [ ] **Step 3: Implement visibility and render integration**

Add the backward-compatible field and helper in `src/view.ts`:

```ts
export type Snapshot = {
  authenticated?: boolean;
  // existing fields remain unchanged
};

export function providerIsVisible(snapshot: Snapshot | undefined): boolean {
  return snapshot?.authenticated !== false;
}
```

In `renderProvider`, set `card.hidden = !providerIsVisible(s)` before rendering. Keep `resizeRosterContent()` at the end so the existing serialized window resize responds to every visibility transition.

- [ ] **Step 4: Run the frontend tests and verify GREEN**

Run: `npm test -- src/view.test.ts`

Expected: all view tests pass.

- [ ] **Step 5: Run full verification**

Run: `npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all frontend and Rust tests pass and the production frontend build succeeds.

- [ ] **Step 6: Commit the frontend change**

```bash
git add src/view.ts src/view.test.ts src/main.ts
git commit -m "feat: hide unauthenticated provider rows"
```
