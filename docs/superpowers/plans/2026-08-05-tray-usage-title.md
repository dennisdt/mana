# Menu-Bar Usage Title Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show remaining usage (session·weekly per provider) as text in the macOS menu bar next to the mana tray icon; widget starts hidden and toggles on left-click.

**Architecture:** A new pure formatter `tray::tray_title` turns the existing poll `Snapshots` map into an `Option<String>`. The poll loop calls `tray.set_title` after every fold. `lib.rs` gives the tray an id, makes left-click toggle the panel (menu moves to right-click), and stops showing the panel at startup.

**Tech Stack:** Rust, Tauri 2.11 (`tray-icon` feature already enabled), tauri-nspanel. Spec: `docs/superpowers/specs/2026-08-05-tray-usage-title-design.md`.

## Global Constraints

- Title format: `C 61·12 X 88·54` — provider letter (`C` Claude, `X` Codex), space, remaining percents joined by `·`; providers joined by a single space; no `%` sign.
- Remaining = `round(100 − used_percent)` clamped to 0–100.
- Only bars with id `session` and `weekly` are shown, in that order; a provider with neither, with empty bars, or unauthenticated is omitted; Claude then Codex order always.
- No qualifying provider → title `None` (icon only).
- All Rust tests run from `src-tauri/` with `cargo test`.
- Never launch the app during tasks; manual smoke test is the user's final step (kill existing instances first per single-instance rule: `pkill -x Mana`).

---

### Task 1: `tray_title` formatter

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs:1-6` (module list only)

**Interfaces:**
- Consumes: `poll::UsageSnapshot` (fields `bars: Vec<parsers::Bar>`, `authenticated: bool`), `parsers::Bar` (fields `id`, `used_percent`).
- Produces: `pub fn tray_title(snapshots: &HashMap<String, UsageSnapshot>) -> Option<String>` in module `tray` — Task 3 calls it as `crate::tray::tray_title(&map)`.

- [ ] **Step 1: Create `src-tauri/src/tray.rs` with failing tests and a stub**

```rust
use std::collections::HashMap;

use crate::poll::UsageSnapshot;

pub fn tray_title(_snapshots: &HashMap<String, UsageSnapshot>) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::Bar;

    fn bar(id: &str, used_percent: f64) -> Bar {
        Bar {
            id: id.into(),
            label: id.into(),
            used_percent,
            resets_at: None,
        }
    }

    fn snap(authenticated: bool, bars: Vec<Bar>) -> UsageSnapshot {
        UsageSnapshot {
            provider: String::new(),
            bars,
            plan: None,
            authenticated,
            status: "ok".into(),
            fetched_at: 0,
        }
    }

    fn map(entries: Vec<(&str, UsageSnapshot)>) -> HashMap<String, UsageSnapshot> {
        entries.into_iter().map(|(k, v)| (k.into(), v)).collect()
    }

    #[test]
    fn both_providers_session_and_weekly() {
        let snapshots = map(vec![
            ("claude", snap(true, vec![bar("session", 39.0), bar("weekly", 88.0)])),
            ("codex", snap(true, vec![bar("session", 12.0), bar("weekly", 46.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 61·12 X 88·54"));
    }

    #[test]
    fn claude_always_precedes_codex() {
        let snapshots = map(vec![
            ("codex", snap(true, vec![bar("session", 0.0)])),
            ("claude", snap(true, vec![bar("session", 0.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 100 X 100"));
    }

    #[test]
    fn single_bar_has_no_separator() {
        let snapshots = map(vec![("claude", snap(true, vec![bar("session", 39.4)]))]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 61"));
    }

    #[test]
    fn model_and_unknown_bars_are_excluded() {
        let snapshots = map(vec![(
            "claude",
            snap(true, vec![bar("model", 10.0), bar("weekly", 50.0)]),
        )]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 50"));
    }

    #[test]
    fn unauthenticated_and_empty_providers_are_omitted() {
        let snapshots = map(vec![
            ("claude", snap(false, vec![bar("session", 10.0)])),
            ("codex", snap(true, vec![])),
        ]);
        assert_eq!(tray_title(&snapshots), None);
    }

    #[test]
    fn empty_map_is_none() {
        assert_eq!(tray_title(&HashMap::new()), None);
    }

    #[test]
    fn out_of_range_percents_clamp() {
        let snapshots = map(vec![(
            "codex",
            snap(true, vec![bar("session", 120.0), bar("weekly", -5.0)]),
        )]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("X 0·100"));
    }
}
```

- [ ] **Step 2: Register the module in `src-tauri/src/lib.rs`**

Add to the module list at the top (after `pub mod progress_store;`):

```rust
pub mod tray;
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tray::`
Expected: FAIL — the stub returns `None`, so every test except `unauthenticated_and_empty_providers_are_omitted` and `empty_map_is_none` fails on assertion.

- [ ] **Step 4: Implement `tray_title`**

Replace the stub in `src-tauri/src/tray.rs`:

```rust
fn remaining(used_percent: f64) -> i64 {
    (100.0 - used_percent).round().clamp(0.0, 100.0) as i64
}

fn provider_segment(letter: &str, snapshot: &UsageSnapshot) -> Option<String> {
    if !snapshot.authenticated {
        return None;
    }
    let find = |id: &str| snapshot.bars.iter().find(|b| b.id == id);
    let numbers: Vec<String> = [find("session"), find("weekly")]
        .into_iter()
        .flatten()
        .map(|b| remaining(b.used_percent).to_string())
        .collect();
    if numbers.is_empty() {
        return None;
    }
    Some(format!("{letter} {}", numbers.join("·")))
}

pub fn tray_title(snapshots: &HashMap<String, UsageSnapshot>) -> Option<String> {
    let segments: Vec<String> = [("claude", "C"), ("codex", "X")]
        .into_iter()
        .filter_map(|(key, letter)| {
            snapshots.get(key).and_then(|s| provider_segment(letter, s))
        })
        .collect();
    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" "))
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tray::`
Expected: 7 passed.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add tray title formatter"
```

---

### Task 2: Tray id, left-click toggle, start hidden

**Files:**
- Modify: `src-tauri/src/lib.rs` (setup fn ~lines 48-113, tests mod)

**Interfaces:**
- Consumes: existing panel/tray setup in `lib.rs`.
- Produces: tray registered with id `"main"` (Task 3 looks it up via `app.tray_by_id("main")`); `fn toggle_widget(app: &tauri::AppHandle)` private to `lib.rs`.

- [ ] **Step 1: Add failing source-inclusion test**

In the `tests` mod of `src-tauri/src/lib.rs` (alongside `builder_registers_activity_store`):

```rust
#[test]
fn tray_toggles_on_left_click_and_widget_starts_hidden() {
    let source = include_str!("lib.rs");
    assert!(source.contains("TrayIconBuilder::with_id(\"main\")"));
    assert!(source.contains(".show_menu_on_left_click(false)"));
    assert!(source.contains(".on_tray_icon_event"));
    assert_eq!(
        source.matches("panel.show()").count(),
        1,
        "panel.show() must appear only inside toggle_widget, not at startup"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml tray_toggles_on_left_click`
Expected: FAIL — `with_id` not present yet.

- [ ] **Step 3: Rewire `lib.rs`**

Update the tray imports (line 9):

```rust
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
```

Add a helper above `pub fn run()` (extracted from the current `"toggle"` menu arm):

```rust
fn toggle_widget(app: &tauri::AppHandle) {
    if let (Some(win), Ok(panel)) =
        (app.get_webview_window("main"), app.get_webview_panel("main"))
    {
        if win.is_visible().unwrap_or(true) {
            panel.hide();
        } else {
            panel.show();
        }
    }
}
```

In `setup`, delete the startup show line entirely — the widget now starts hidden; the panel stays configured but is not ordered front:

```rust
panel.show(); // orderFrontRegardless — no activation
```

Replace the tray construction:

```rust
TrayIconBuilder::with_id("main")
    .icon(tray_template_icon()?)
    .icon_as_template(true)
    .tooltip("Mana")
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_menu_event(|app, event| match event.id().as_ref() {
        "toggle" => toggle_widget(app),
        "quit" => app.exit(0),
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            toggle_widget(tray.app_handle());
        }
    })
    .build(app)?;
```

- [ ] **Step 4: Run the full Rust suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all pass, including the new source test and the pre-existing `vibrancy`/`tray_template` tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: left-click tray toggle, start widget hidden"
```

---

### Task 3: Poll loop sets the tray title

**Files:**
- Modify: `src-tauri/src/poll.rs:174-188` (inside `spawn_pollers` loop), tests mod

**Interfaces:**
- Consumes: `crate::tray::tray_title` (Task 1), tray id `"main"` (Task 2).
- Produces: tray title refreshed after every poll fold.

- [ ] **Step 1: Add failing source-inclusion test**

In the `tests` mod of `src-tauri/src/poll.rs`:

```rust
#[test]
fn poll_loop_refreshes_tray_title() {
    let source = include_str!("poll.rs");
    assert!(source.contains("tray::tray_title"));
    assert!(source.contains("tray_by_id(\"main\")"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml poll_loop_refreshes`
Expected: FAIL.

- [ ] **Step 3: Update the poll loop**

In `spawn_pollers`, extend the existing lock scope to also compute the title, then set it after the `emit`:

```rust
let (next, title) = {
    let state = app.state::<Snapshots>();
    let mut map = state.lock().unwrap();
    let next = fold_snapshot(map.get(provider), provider, result, epoch_now());
    map.insert(provider.to_string(), next.clone());
    (next, crate::tray::tray_title(&map))
};
eprintln!(
    "[mana] {} {} bars={}",
    provider,
    next.status,
    next.bars.len()
);
let _ = app.emit("usage-update", &next);
if let Some(tray) = app.tray_by_id("main") {
    let _ = tray.set_title(title.as_deref());
}
```

(`tray_by_id` miss and `set_title` errors are deliberately ignored — next tick retries, per spec.)

- [ ] **Step 4: Run the full Rust suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all pass.

- [ ] **Step 5: Run the frontend suite (regression only — no frontend changes)**

Run: `npm test`
Expected: all pass, unchanged.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/poll.rs
git commit -m "feat: show remaining usage in menu-bar tray title"
```

---

### Final verification (user-run)

- [ ] `pkill -x Mana` (single-instance rule), then `npm run tauri dev`; confirm: widget does not appear at launch, title like `C 61·12 X 88·54` appears within ~1 minute, left-click toggles the widget, right-click shows the menu.
