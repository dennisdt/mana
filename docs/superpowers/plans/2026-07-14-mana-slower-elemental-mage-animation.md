# Slower Elemental Mage Animation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Slow the four-frame Claude and Codex animation rows to half their current playback speed while preserving state hierarchy and reduced-motion behavior.

**Architecture:** `src/sprite-animation.ts` owns frame duration and frame selection for both mage atlases. The focused Vitest suite asserts durations and every frame boundary, so changing expectations first proves the pacing update before production code changes.

**Tech Stack:** TypeScript, Vitest, Vite.

## Global Constraints

- Keep the existing four-frame atlas geometry and DOM `data-frame` renderer unchanged.
- Set idle, working, and hover frame durations to exactly `575`, `340`, and `410` milliseconds.
- Preserve working as fastest and idle as slowest.
- Keep `prefers-reduced-motion` behavior frozen on frame zero.
- Do not modify sprite assets, state listeners, layout, or release version metadata.

---

### Task 1: Double Elemental Mage Frame Durations

**Files:**
- Modify: `src/sprite-animation.test.ts:6-29`
- Modify: `src/sprite-animation.ts:1-5`

**Interfaces:**
- Consumes: `SPRITE_FRAME_DURATION_MS` and `spriteFrameAt(elapsedMs, state, reducedMotion)`.
- Produces: four-frame loops that advance at `575ms` idle, `340ms` working, and `410ms` hover, with the existing idle fallback for unknown states.

- [ ] **Step 1: Write the failing timing expectations**

In `src/sprite-animation.test.ts`, replace the parameter table with:

```ts
  it.each([
    ["idle", 575],
    ["working", 340],
    ["hover", 410],
  ] as const)("advances the %s row through four frames", (state, frameDuration) => {
```

Update the invalid-state expectations to:

```ts
  it("uses idle timing for missing or invalid DOM state", () => {
    expect(spriteFrameAt(575, undefined, false)).toBe(1);
    expect(spriteFrameAt(575, "unknown", false)).toBe(1);
  });
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run: `npm test -- src/sprite-animation.test.ts`

Expected: FAIL because the current implementation still exposes `287.5`, `170`, and `205` milliseconds and advances fallback idle state at `287.5ms`.

- [ ] **Step 3: Update the centralized duration map**

In `src/sprite-animation.ts`, replace the duration map with:

```ts
export const SPRITE_FRAME_DURATION_MS = {
  idle: 575,
  working: 340,
  hover: 410,
} as const;
```

Do not change `frameDuration`, `spriteFrameAt`, or `SPRITE_TICK_MS`.

- [ ] **Step 4: Run focused and complete frontend verification**

Run: `npm test -- src/sprite-animation.test.ts`

Expected: PASS for frame boundaries, reduced-motion freeze, invalid-state fallback, and legacy WebKit listener coverage.

Run: `npm test && npm run build && git diff --check`

Expected: frontend tests pass, TypeScript/Vite build succeeds, and diff check emits no whitespace errors.

- [ ] **Step 5: Commit the pacing implementation**

Run:

```bash
git add src/sprite-animation.ts src/sprite-animation.test.ts
git commit -m "feat: slow elemental mage animations"
```
