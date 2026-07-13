# mana Party Roster UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship mana v0.3.0 with a readable glass Party Roster HUD, provider-owned familiars, content-fitted expansion, and duration-correct Codex labels.

**Architecture:** Keep the existing Tauri 2 and vanilla TypeScript architecture. Isolate pure HTML rendering in `src/view.ts` and pure hover/window calculations in `src/window-layout.ts` so behavior is covered by Vitest, while the Rust parser classifies Codex windows by their declared duration. `src/main.ts` remains the integration layer for Tauri events, DOM updates, animation state, and serialized native-window resizing.

**Tech Stack:** Tauri 2, Rust 2021, vanilla TypeScript 5.6, CSS, Vite 6, Vitest 3, native macOS `HudWindow` vibrancy.

## Global Constraints

- Collapsed geometry remains exactly `340x48` logical pixels.
- Expanded width is exactly `420` logical pixels and expanded height is measured from rendered content.
- Mouse leave collapses after exactly `150ms`; re-entry cancels the pending collapse.
- Codex `18000`-second windows are `session` / `5 hour`; `604800`-second windows are `weekly` / `Weekly`; unknown durations use neutral field-based identities.
- Claude session labels render as `5 hour`; scoped model labels such as `Fable` remain unchanged.
- Reuse the existing Clawd and Nimbus sprite sheets and their idle, working, and hover rows.
- Keep native `HudWindow` vibrancy, polling cadence, endpoints, credential reads, token safety, process detection, and position persistence unchanged.
- Add no runtime dependencies, UI framework, new endpoint, new credential behavior, sound, setting, notification, or new mascot art.
- All motion, including the working mana scan, must stop under `prefers-reduced-motion: reduce`.

---

## File Structure

- Create `src/view.ts`: normalized frontend snapshot types and pure pill/provider HTML renderers.
- Create `src/view.test.ts`: renderer behavior, escaping, weekly-only Codex, and absent-state coverage.
- Create `src/window-layout.ts`: geometry constants, edge-safe expansion calculation, measured-height helper, and hover-intent controller.
- Create `src/window-layout.test.ts`: geometry and delayed-collapse behavior.
- Create `src-tauri/tests/fixtures/codex_pro_weekly.json`: representative weekly-only Codex Pro response.
- Modify `src-tauri/src/parsers.rs`: duration-based Codex classification and normalized Claude session label.
- Modify `src/main.ts`: consume the pure renderers, update every provider sprite, and integrate dynamic/edge-safe hover expansion.
- Modify `index.html`: provider metadata for pill sprites and semantic provider classes for expanded sections.
- Modify `src/styles.css`: glass treatment, Party Roster grid, readable typography, provider activity signal, working scan, and reduced-motion rules.
- Modify `README.md`: describe duration-aware Codex limits and provider-owned expanded familiars.
- Modify `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`: synchronize v0.3.0.

---

### Task 1: Classify Codex windows by duration

**Files:**
- Create: `src-tauri/tests/fixtures/codex_pro_weekly.json`
- Modify: `src-tauri/src/parsers.rs:17-80`
- Test: `src-tauri/src/parsers.rs:83-143`

**Interfaces:**
- Consumes: Codex `rate_limit.primary_window` and `secondary_window` JSON objects.
- Produces: `parse_codex(&serde_json::Value) -> (Vec<Bar>, Option<String>)` with stable semantic ids and truthful labels.

- [ ] **Step 1: Add the weekly-only Pro fixture**

```json
{
  "plan_type": "pro",
  "rate_limit": {
    "allowed": true,
    "limit_reached": false,
    "primary_window": {
      "used_percent": 55,
      "limit_window_seconds": 604800,
      "reset_after_seconds": 345600,
      "reset_at": 1784487600
    },
    "secondary_window": null
  }
}
```

- [ ] **Step 2: Write failing parser tests**

Update the existing Claude and Pro Lite expectations to `5 hour`, then add:

```rust
#[test]
fn codex_pro_weekly_primary() {
    let (bars, plan) = parse_codex(&load(include_str!(
        "../tests/fixtures/codex_pro_weekly.json"
    )));
    assert_eq!(plan.as_deref(), Some("pro"));
    assert_eq!(bars, vec![Bar {
        id: "weekly".into(),
        label: "Weekly".into(),
        used_percent: 55.0,
        resets_at: Some(1784487600),
    }]);
}

#[test]
fn codex_unknown_duration_uses_neutral_identity() {
    let (bars, _) = parse_codex(&load(r#"{
        "rate_limit": {
            "primary_window": {
                "used_percent": 12,
                "limit_window_seconds": 86400,
                "reset_at": 1784487600
            }
        }
    }"#));
    assert_eq!(bars[0].id, "primary");
    assert_eq!(bars[0].label, "Primary");
}
```

- [ ] **Step 3: Run the focused tests and verify RED**

Run:

```bash
cd src-tauri
cargo test parsers::tests::codex_
```

Expected: FAIL because `primary_window` is still hardcoded as `session` / `5h`.

- [ ] **Step 4: Implement duration-based identities**

Add this helper above `parse_codex` and use it inside the existing two-window loop:

```rust
fn codex_window_identity(key: &str, window: &serde_json::Value) -> (&'static str, &'static str) {
    match window.get("limit_window_seconds").and_then(|v| v.as_i64()) {
        Some(18_000) => ("session", "5 hour"),
        Some(604_800) => ("weekly", "Weekly"),
        _ if key == "primary_window" => ("primary", "Primary"),
        _ => ("secondary", "Secondary"),
    }
}
```

Change both Claude session label sites from `5h` to `5 hour`. In `parse_codex`, iterate only the field keys, call `codex_window_identity(key, w)`, and copy the returned strings into each `Bar`.

- [ ] **Step 5: Run parser and full Rust tests and verify GREEN**

Run:

```bash
cd src-tauri
rustfmt --edition 2021 src/parsers.rs
cargo test parsers::tests
cargo test
```

Expected: all parser tests and all Rust tests PASS.

- [ ] **Step 6: Commit the parser fix**

```bash
git add src-tauri/tests/fixtures/codex_pro_weekly.json src-tauri/src/parsers.rs
git commit -m "fix: classify Codex limits by duration"
```

---

### Task 2: Extract and test Party Roster markup

**Files:**
- Create: `src/view.ts`
- Create: `src/view.test.ts`
- Modify: `src/main.ts:4-126`
- Modify: `index.html:11-22`

**Interfaces:**
- Consumes: `Snapshot`, provider id (`claude` or `codex`), and existing `manaLeft`.
- Produces: `pillHtml(snapshot, provider)` and `cardHtml(snapshot, provider)` strings whose dynamic nodes retain `.track`, `.pct`, `.cd`, `.plan`, and `.age` hooks used by `applyData`.

- [ ] **Step 1: Write failing renderer tests**

Create `src/view.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import { cardHtml, pillHtml, type Snapshot } from "./view";

const weeklyOnly: Snapshot = {
  provider: "codex",
  plan: "pro",
  status: "ok",
  fetched_at: 1_784_487_600,
  bars: [{ id: "weekly", label: "Weekly", used_percent: 55, resets_at: 1_784_487_600 }],
};

describe("cardHtml", () => {
  it("renders a provider-owned familiar and weekly-only row", () => {
    const html = cardHtml(weeklyOnly, "codex");
    expect(html).toContain('class="sprite nimbus"');
    expect(html).toContain('data-provider="codex"');
    expect(html).toContain('class="lbl">Weekly</span>');
    expect(html).not.toContain("5 hour");
  });

  it("keeps the provider roster shell when data is absent", () => {
    const html = cardHtml(undefined, "claude");
    expect(html).toContain('class="sprite clawd"');
    expect(html).toContain("Claude");
    expect(html).toContain("log in via the claude CLI");
  });

  it("escapes limit labels", () => {
    const snapshot = { ...weeklyOnly, bars: [{ ...weeklyOnly.bars[0], label: '<script>' }] };
    expect(cardHtml(snapshot, "codex")).toContain("&#60;script&#62;");
  });
});

describe("pillHtml", () => {
  it("uses the first available bar when no session bar exists", () => {
    expect(pillHtml(weeklyOnly, "codex")).toContain('data-bar="0"');
  });
});
```

- [ ] **Step 2: Run the renderer test and verify RED**

Run:

```bash
npx vitest run src/view.test.ts
```

Expected: FAIL because `src/view.ts` does not exist.

- [ ] **Step 3: Create the pure renderer module**

Create `src/view.ts` with the existing normalized types and renderer hooks:

```typescript
import { manaLeft } from "./format";

export type Bar = {
  id: string;
  label: string;
  used_percent: number;
  resets_at: number | null;
};

export type Snapshot = {
  provider: string;
  bars: Bar[];
  plan: string | null;
  status: string;
  fetched_at: number;
};

const GEMS: Record<string, string> = { claude: "◆", codex: "●" };

function esc(value: string): string {
  return value.replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);
}

function spriteHtml(provider: string): string {
  const className = provider === "claude" ? "clawd" : "nimbus";
  return `<div class="sprite ${className}" data-provider="${provider}" data-state="idle" aria-hidden="true"></div>`;
}

function barHtml(snapshot: Snapshot, bar: Bar, index: number): string {
  const left = manaLeft(bar.used_percent);
  return `<div class="track ${snapshot.provider}${left < 30 ? " low" : ""}" data-bar="${index}"><div class="fill" style="width:${left}%"></div></div>`;
}

export function pillHtml(snapshot: Snapshot | undefined, provider: string): string {
  if (!snapshot || snapshot.status === "absent" || snapshot.bars.length === 0) {
    return `<span class="gem">${GEMS[provider]}</span><span class="nums">no data</span>`;
  }
  const sessionIndex = snapshot.bars.findIndex((bar) => bar.id === "session");
  const index = sessionIndex >= 0 ? sessionIndex : 0;
  return `<span class="gem ${snapshot.provider}">${GEMS[provider]}</span>
    ${barHtml(snapshot, snapshot.bars[index], index)}
    <span class="nums"><b class="pct" data-bar="${index}"></b><span class="cd" data-bar="${index}"></span></span>`;
}

export function cardHtml(snapshot: Snapshot | undefined, provider: string): string {
  const name = provider === "claude" ? "Claude" : "Codex";
  const hasData = snapshot && snapshot.status !== "absent" && snapshot.bars.length > 0;
  const rows = hasData
    ? `<div class="rows">${snapshot.bars.map((bar, index) => `<div class="row">
        <span class="lbl">${esc(bar.label)}</span>
        ${barHtml(snapshot, bar, index)}
        <span class="val"><b class="pct" data-bar="${index}"></b><span class="cd" data-bar="${index}"></span></span>
      </div>`).join("")}</div>`
    : `<div class="empty">no data - log in via the ${provider} CLI</div>`;
  return `<div class="familiar-slot">${spriteHtml(provider)}</div>
    <div class="provider-content">
      <div class="head"><strong>${name}</strong><span class="plan"></span><span class="activity-signal" aria-hidden="true"></span><span class="age"></span></div>
      ${rows}
    </div>`;
}
```

- [ ] **Step 4: Integrate the renderers and provider sprite metadata**

In `src/main.ts`:

- Import `cardHtml`, `pillHtml`, and `type Snapshot` from `./view`.
- Delete the local `Bar`, `Snapshot`, `GEMS`, `esc`, `barHtml`, `pillHtml`, and `cardHtml` declarations.
- Change `updateSprites` to update every matching provider sprite:

```typescript
document
  .querySelectorAll<HTMLElement>(`.sprite[data-provider="${provider}"]`)
  .forEach((element) => {
    element.dataset.state = spriteState(provider);
  });
```

In `index.html`, replace pill sprite ids with `data-provider="claude"` and `data-provider="codex"`, and add `class="provider-card claude"` / `class="provider-card codex"` to the two expanded sections.

- [ ] **Step 5: Run renderer tests, the frontend suite, and the build**

Run:

```bash
npx vitest run src/view.test.ts
npm test
npm run build
```

Expected: renderer tests PASS, the full frontend suite passes, and TypeScript/Vite build exits 0.

- [ ] **Step 6: Commit the testable roster markup**

```bash
git add src/view.ts src/view.test.ts src/main.ts index.html
git commit -m "feat: render provider-owned roster familiars"
```

---

### Task 3: Add tested hover intent and edge-safe content sizing

**Files:**
- Create: `src/window-layout.ts`
- Create: `src/window-layout.test.ts`
- Modify: `src/main.ts:1-23,128-170`

**Interfaces:**
- Consumes: physical window origin, physical monitor work area, monitor scale factor, rendered card scroll height, and hover entry/exit events.
- Produces: `expandedOrigin`, `expandedHeight`, and `createHoverIntent`; `src/main.ts` maps their plain values to Tauri `LogicalSize` and `PhysicalPosition` instances.

- [ ] **Step 1: Write failing geometry and hover tests**

Create `src/window-layout.test.ts`:

```typescript
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  COLLAPSE_DELAY_MS,
  createHoverIntent,
  expandedHeight,
  expandedOrigin,
} from "./window-layout";

afterEach(() => vi.useRealTimers());

describe("expandedOrigin", () => {
  it("keeps an origin that fits the active work area", () => {
    expect(expandedOrigin({ x: 500, y: 80 }, { x: 0, y: 0, width: 2880, height: 1800 }, 2))
      .toEqual({ x: 500, y: 80 });
  });

  it("shifts left only enough to keep 420 logical pixels visible", () => {
    expect(expandedOrigin({ x: 2100, y: 80 }, { x: 0, y: 0, width: 2880, height: 1800 }, 2))
      .toEqual({ x: 2040, y: 80 });
  });
});

it("rounds measured card height plus the root border", () => {
  expect(expandedHeight(207.2)).toBe(210);
});

it("delays collapse and cancels it on re-entry", () => {
  vi.useFakeTimers();
  const hoverStates: boolean[] = [];
  const expandedStates: boolean[] = [];
  const intent = createHoverIntent(
    (value) => hoverStates.push(value),
    (value) => expandedStates.push(value),
  );
  intent.enter();
  intent.leave();
  vi.advanceTimersByTime(COLLAPSE_DELAY_MS - 1);
  expect(expandedStates).toEqual([true]);
  intent.enter();
  vi.advanceTimersByTime(COLLAPSE_DELAY_MS);
  expect(hoverStates).toEqual([true, false, true]);
  expect(expandedStates).toEqual([true, true]);
});
```

- [ ] **Step 2: Run the window-layout test and verify RED**

Run:

```bash
npx vitest run src/window-layout.test.ts
```

Expected: FAIL because `src/window-layout.ts` does not exist.

- [ ] **Step 3: Implement the pure layout and hover helpers**

Create `src/window-layout.ts`:

```typescript
export const COLLAPSED_WIDTH = 340;
export const COLLAPSED_HEIGHT = 48;
export const EXPANDED_WIDTH = 420;
export const COLLAPSE_DELAY_MS = 150;

type Point = { x: number; y: number };
type Rect = Point & { width: number; height: number };

export function expandedOrigin(origin: Point, workArea: Rect, scaleFactor: number): Point {
  const maximumX = workArea.x + workArea.width - EXPANDED_WIDTH * scaleFactor;
  return { x: Math.max(workArea.x, Math.min(origin.x, maximumX)), y: origin.y };
}

export function expandedHeight(cardScrollHeight: number): number {
  return Math.ceil(cardScrollHeight + 2);
}

export function createHoverIntent(
  setHovering: (value: boolean) => void,
  setExpanded: (value: boolean) => void,
) {
  let collapseTimer: ReturnType<typeof setTimeout> | undefined;
  return {
    enter(): void {
      clearTimeout(collapseTimer);
      collapseTimer = undefined;
      setHovering(true);
      setExpanded(true);
    },
    leave(): void {
      setHovering(false);
      clearTimeout(collapseTimer);
      collapseTimer = setTimeout(() => setExpanded(false), COLLAPSE_DELAY_MS);
    },
  };
}
```

- [ ] **Step 4: Replace fixed expansion with serialized measured expansion**

In `src/main.ts`:

- Import `currentMonitor`, `PhysicalPosition`, and the constants/helpers from `./window-layout`.
- Build `COLLAPSED` from the exported dimensions and remove the fixed `EXPANDED` size.
- Track `expanded`, `collapsedOrigin`, and `expandedOffsetX` separately from `hovering`.
- On expansion: store the current physical origin, add `body.expanded`, wait one animation frame, measure `#card.scrollHeight`, calculate the monitor-safe origin, set `LogicalSize(EXPANDED_WIDTH, expandedHeight(...))`, and set the calculated physical position.
- On collapse: remove `body.expanded`, restore `COLLAPSED`, restore `collapsedOrigin`, and clear the saved origin/offset.
- Keep all size/position work on the existing `sizing` promise chain.
- Replace direct mouse handlers with `createHoverIntent`; its hover callback updates `hovering` and sprites, and its expansion callback invokes `setExpanded`.
- After `renderProvider`, enqueue a content-height resize when `expanded` is true.
- In the existing 300ms `onMoved` completion callback, if expanded, recompute `collapsedOrigin` from the actual expanded position minus `expandedOffsetX`; this preserves a user drag while restoring the pill's aligned position on collapse.

The expansion body uses this ordering:

```typescript
document.body.classList.add("expanded");
await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
const origin = await win.outerPosition();
const monitor = await currentMonitor();
const target = monitor
  ? expandedOrigin(origin, {
      x: monitor.workArea.position.x,
      y: monitor.workArea.position.y,
      width: monitor.workArea.size.width,
      height: monitor.workArea.size.height,
    }, monitor.scaleFactor)
  : origin;
collapsedOrigin = origin;
expandedOffsetX = target.x - origin.x;
await win.setSize(new LogicalSize(EXPANDED_WIDTH, expandedHeight(card.scrollHeight)));
await win.setPosition(new PhysicalPosition(target.x, target.y));
```

- [ ] **Step 5: Run layout tests, full frontend tests, and build**

Run:

```bash
npx vitest run src/window-layout.test.ts
npm test
npm run build
```

Expected: layout tests PASS, full frontend suite passes, and build exits 0.

- [ ] **Step 6: Commit window behavior**

```bash
git add src/window-layout.ts src/window-layout.test.ts src/main.ts
git commit -m "feat: fit and stabilize roster expansion"
```

---

### Task 4: Apply the glass HUD, verify the native app, and release v0.3.0

**Files:**
- Modify: `src/styles.css`
- Modify: `README.md:1-28`
- Modify: `package.json:4`
- Modify: `package-lock.json:1-10`
- Modify: `src-tauri/Cargo.toml:3`
- Modify: `src-tauri/Cargo.lock` (`mana` package entry only, via Cargo)
- Modify: `src-tauri/tauri.conf.json:4`

**Interfaces:**
- Consumes: the provider-card markup from Task 2 and expanded/body state from Task 3.
- Produces: the approved visual system and a synchronized v0.3.0 release bundle.

- [ ] **Step 1: Record the current visual failures**

Run the current native app and retain the user-supplied screenshot as the RED baseline. Confirm before editing CSS that the current expanded state has all four failures: clipped reset text, duplicated summary/detail, detached expanded mascots, and a flat opaque panel.

- [ ] **Step 2: Implement the approved CSS token system**

Replace the visual tokens and expanded layout using these binding values:

```css
:root {
  --void: #080d1a;
  --ink: #edf3ff;
  --ink-dim: #a8b5cc;
  --claude-1: #48d6ff;
  --claude-2: #6f7cff;
  --codex-1: #c865ff;
  --codex-2: #e45ab7;
}

#root {
  position: relative;
  box-sizing: border-box;
  min-height: 100%;
  overflow: hidden;
  border: 1px solid rgba(147, 184, 255, 0.38);
  border-radius: 16px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.10), transparent 18%),
    linear-gradient(135deg, rgba(18, 30, 53, 0.42), rgba(8, 13, 26, 0.62));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.16),
    inset 0 0 38px rgba(91, 119, 195, 0.10);
}

body.expanded #pill { display: none; }

body.expanded #card {
  display: flex;
}

#card {
  flex-direction: column;
  padding: 14px 16px;
}

#card section {
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr);
  gap: 11px;
  padding: 5px 0 13px;
}

#card section + section {
  border-top: 1px solid rgba(160, 188, 235, 0.16);
  padding-top: 14px;
  padding-bottom: 4px;
}

.rows { display: grid; gap: 7px; }

.row {
  display: grid;
  grid-template-columns: 52px minmax(82px, 1fr) max-content;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
```

Add `#root::before` for the fading 8px pixel grid and `#root::after` for one top-edge highlight. Keep pseudo-elements pointer-free and behind content. Make provider names 13px bold monospace, row labels/reset metadata at least 11px, percentages 12px semibold, and retain tabular numerals. Style `.activity-signal` from each section's provider variables. Add the working scan only under `.provider-card:has(.sprite[data-state="working"]) .fill::after` or an equivalent class-based selector. Extend reduced-motion rules to disable the scan.

- [ ] **Step 3: Synchronize version 0.3.0 and README copy**

Run:

```bash
npm version 0.3.0 --no-git-tag-version
```

Set `version = "0.3.0"` in `src-tauri/Cargo.toml` and `"version": "0.3.0"` in `src-tauri/tauri.conf.json`, then run `cargo check` in `src-tauri` to update only the local `mana` package entry in `Cargo.lock`.

Update the README summary and Familiars section to state that expanded mode is a familiar-led roster and that Codex limits are named from their actual duration, including weekly-only Pro.

- [ ] **Step 4: Run the complete automated verification suite**

Run:

```bash
npm test
npm run build
cd src-tauri
rustfmt --edition 2021 --check src/parsers.rs
cargo test
cargo check
```

Expected: all frontend tests pass, Vite builds, Rust formatting is clean, all Rust tests pass, and Cargo check exits 0.

- [ ] **Step 5: Verify the native Tauri window visually and behaviorally**

Run `npm run tauri dev`, then inspect the real native window at Retina scale:

- Collapsed state is `340x48`, glass is visible, both summaries fit, and each mascot remains next to its provider.
- Expanded state hides the pill, is 420px wide, fits its content height, and shows full weekday reset strings.
- Claude with three rows plus Codex with two rows has no clipping, overlap, scrollbar, or large blank region.
- Weekly-only Codex Pro reads `Weekly`, not `5 hour`.
- Pointer exit waits 150ms; re-entry cancels collapse.
- Expansion stays within the active monitor work area and collapse restores the aligned pill position.
- Idle, working, hover, low-mana, stale, absent, and reduced-motion states retain stable geometry.
- Browser/console and native logs contain no new errors.

Capture collapsed and expanded screenshots and inspect them before proceeding.

- [ ] **Step 6: Build and install the release app**

Run:

```bash
npm run tauri build
```

Stop the currently running mana instance, replace `/Applications/mana.app` with `src-tauri/target/release/bundle/macos/mana.app` using a reversible backup, launch the new app, and repeat the collapsed/expanded smoke check against the installed binary.

At action time, ask for permission before quitting the running installed app.
After approval, use `cua-driver hotkey` with `cmd+q`; do not use `kill`,
`killall`, or `pkill`. Then use these replacement commands:

```bash
backup="/tmp/mana.app.v0.2.1-backup-$(date +%s)"
mv /Applications/mana.app "$backup"
if ditto src-tauri/target/release/bundle/macos/mana.app /Applications/mana.app; then
  printf 'Previous app backup: %s\n' "$backup"
else
  mv "$backup" /Applications/mana.app
  exit 1
fi
```

Launch the installed build without foregrounding it:

```bash
cua-driver launch_app '{"bundle_id":"com.vantasoft.mana"}'
```

Do not use any form of the `open` command for app launch.

- [ ] **Step 7: Commit the visual release**

```bash
git add src/styles.css README.md package.json package-lock.json \
  src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "feat: ship the glass party roster HUD"
```

---

## Final Review

- [ ] Review the complete diff against `docs/superpowers/specs/2026-07-13-mana-party-roster-ux-design.md`.
- [ ] Confirm no unrelated files, `.DS_Store`, temporary screenshots, or `.superpowers` artifacts are staged.
- [ ] Re-run `npm test`, `npm run build`, `rustfmt --edition 2021 --check src/parsers.rs`, `cargo test`, and `cargo check` after all review fixes. Full-repo `cargo fmt --check` has pre-existing failures outside this change and is not a release gate for v0.3.0.
- [ ] Confirm the installed `/Applications/mana.app` reports and renders v0.3.0 behavior.
