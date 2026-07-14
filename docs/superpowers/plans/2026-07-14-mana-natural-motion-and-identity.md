# Mana Natural Motion and Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stagger the existing relaxed provider animations and ship the approved fantasy mana-potion artwork as the version 0.4.4 `Mana` app icon.

**Architecture:** Keep animation timing deterministic by applying provider and row phase offsets to the existing JavaScript scheduler and CSS keyframes. Preserve the approved image as a canonical master, let Tauri derive all platform assets, and change only user-facing product metadata while retaining lowercase internal package names and the existing bundle identifier.

**Tech Stack:** TypeScript, Vitest, CSS, Tauri 2, Rust, macOS app bundles

## Global Constraints

- The visible product name is exactly `Mana`; internal npm package and Rust crate names remain lowercase `mana`.
- The bundle identifier remains `com.vantasoft.mana`.
- Release version is exactly `0.4.4` in npm, Cargo, and Tauri metadata.
- Canonical icon source is `src-tauri/icons/mana-potion-master.png`, copied byte-for-byte from the user-approved 1254x1254 RGB artwork.
- Preserve the icon's opaque blue-sky and periwinkle-cloud fantasy background.
- Keep current sprite assets, mana-bar assets, layout, colors, polling, activity detection, window behavior, durations, and easing unchanged.
- Reduced motion freezes sprites on frame zero and disables glints and activity pulses without scheduling timers.

---

### Task 1: Deterministic Motion Phases

**Files:**
- Modify: `src/sprite-animation.ts`
- Modify: `src/sprite-animation.test.ts`
- Modify: `src/main.ts`
- Modify: `src/styles.css`
- Modify: `src/styles.test.ts`

**Interfaces:**
- Produces: `spritePhaseCycles(provider: string | undefined): number` returning `0.375` for Codex and `0` otherwise.
- Produces: optional `phaseCycles` parameter on `spriteFrameAt` and `spriteFrameDelayAt`, defaulting to zero.
- Consumes: existing `data-provider` attributes on provider sprite elements and existing provider card IDs.

- [ ] **Step 1: Write failing phase tests**

Add table-driven tests showing that Claude and Codex choose different frames at timestamp zero in idle, working, and hover states; that both scheduling functions use the same phase-adjusted clock; and that unknown providers return zero. Add stylesheet assertions for `--provider-motion-offset`, `--row-motion-offset`, `calc(var(--provider-motion-offset) + var(--row-motion-offset))`, and provider-only activity delay.

- [ ] **Step 2: Run focused tests and confirm the new assertions fail**

Run: `npm test -- --run src/sprite-animation.test.ts src/styles.test.ts`

Expected: FAIL because provider phases and CSS delay variables do not exist yet.

- [ ] **Step 3: Implement provider-aware sprite timing**

Define `spritePhaseCycles`, add a defaulted `phaseCycles = 0` argument to both timing functions, and calculate `phaseAdjustedElapsed = elapsed + frameDuration(state) * 4 * phaseCycles`. Use that exact value for frame selection and next-boundary delay. In `src/main.ts`, derive each sprite's phase from `element.dataset.provider` for both calls.

- [ ] **Step 4: Implement deterministic CSS delays**

Set provider offsets to `0s` for Claude and `-0.8s` for Codex. Set row offsets to `0s`, `-0.85s`, `-1.7s`, and `-2.55s`. Apply the sum to mana glints and only the provider offset to activity pulses; retain the existing reduced-motion override.

- [ ] **Step 5: Run focused and complete frontend tests**

Run: `npm test -- --run src/sprite-animation.test.ts src/styles.test.ts && npm test`

Expected: all focused tests and the complete frontend suite PASS.

- [ ] **Step 6: Commit**

```bash
git add src/sprite-animation.ts src/sprite-animation.test.ts src/main.ts src/styles.css src/styles.test.ts
git commit -m "fix: stagger provider motion phases"
```

### Task 2: Mana Product Identity and Icon Set

**Files:**
- Create: `src-tauri/icons/mana-potion-master.png`
- Regenerate: `src-tauri/icons/**`
- Create: `src/branding.test.ts`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `index.html`
- Modify: `README.md`

**Interfaces:**
- Produces: `Mana.app` version `0.4.4` with bundle identifier `com.vantasoft.mana`.
- Produces: one canonical icon source and the standard Tauri-generated PNG, ICNS, ICO, iOS, Android, and Windows icon assets.

- [ ] **Step 1: Add failing branding tests**

Add `src/branding.test.ts` assertions that parse Tauri/package/Cargo metadata and inspect HTML/README text. Require visible name `Mana`, synchronized version `0.4.4`, unchanged internal name and identifier, the canonical icon path, and a square 1254x1254 PNG master.

- [ ] **Step 2: Run the branding test and confirm failure**

Run: `npm test -- --run src/branding.test.ts`

Expected: FAIL because metadata is still `0.4.3`, the visible name is lowercase, and the canonical master is not in the repository.

- [ ] **Step 3: Persist the approved icon and regenerate platform assets**

Copy the approved generated source byte-for-byte to `src-tauri/icons/mana-potion-master.png`. Run `npm run tauri -- icon src-tauri/icons/mana-potion-master.png` and retain the canonical source alongside the generated assets.

- [ ] **Step 4: Synchronize user-facing identity and version metadata**

Set Tauri `productName` and window `title`, the HTML title, and README product references to `Mana`. Set version `0.4.4` in Tauri, npm lockfiles, and Cargo lockfiles. Keep npm/Cargo package names lowercase and retain `com.vantasoft.mana`.

- [ ] **Step 5: Verify tests, icon dimensions, and production builds**

Run: `npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml && npm run tauri build`

Expected: frontend and Rust suites PASS; production frontend and native bundle builds succeed at `src-tauri/target/release/bundle/macos/Mana.app`.

Inspect: `file src-tauri/icons/mana-potion-master.png src-tauri/icons/32x32.png src-tauri/icons/128x128.png src-tauri/icons/icon.icns` and visually inspect the master plus generated 32px, 128px, and packaged icon assets.

- [ ] **Step 6: Install and verify the native bundle**

Preserve any existing installed app as a timestamped `/tmp` backup, copy the verified bundle to `/Applications/Mana.app`, then compare SHA-256 hashes of the built and installed executables. Inspect `Info.plist` to confirm `Mana`, `com.vantasoft.mana`, and `0.4.4`. Do not terminate the currently running app without explicit restart permission.

- [ ] **Step 7: Commit**

```bash
git add README.md index.html package.json package-lock.json src/branding.test.ts src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json src-tauri/icons
git commit -m "feat: ship Mana potion app icon"
```

### Task 3: Final Integrated Review

**Files:**
- Review: all changes since the plan baseline

**Interfaces:**
- Consumes: completed Tasks 1 and 2.
- Produces: evidence that the release satisfies the complete approved specification.

- [ ] **Step 1: Run the complete verification matrix**

Run: `npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: every test and build command exits zero.

- [ ] **Step 2: Review the complete diff**

Check for accidental package/crate renames, identifier drift, hand-edited derived icon inconsistencies, synchronized animation regressions, missing reduced-motion behavior, and unrelated layout or asset changes.

- [ ] **Step 3: Confirm repository state**

Run: `git status --short && git log --oneline -5`

Expected: only the pre-existing unrelated `.DS_Store` remains untracked; implementation and review commits are present.
