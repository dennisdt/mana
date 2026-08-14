# Menu-bar usage title

2026-08-05 · approved by Dennis

## Goal

Show remaining usage in the macOS menu bar next to the mana tray icon (like the
battery percentage), so the widget window can stay hidden until toggled.

## Behavior

- Tray title shows remaining percent per authenticated provider:
  `✳ 61·12 ⎔ 54` — Claude glyph `✳` with weekly·model, Codex glyph `⎔` with
  weekly (Claude, then Codex, joined by a single space).
- Remaining = `round(100 − used_percent)`, clamped to 0–100. No `%` sign.
- Bars are selected by id per provider: Claude `weekly` then `model`
  (model = the model-scoped weekly limit, e.g. Fable); Codex `weekly`. If
  only one listed bar exists, show that single number without `·`. Session
  bars are excluded.
- A provider with no usable bars (unauthenticated, or `absent` with empty
  bars) is omitted. If no provider qualifies, the title is cleared — icon only.
- Stale snapshots display their last values as-is (matches widget behavior).
- Title updates after every poll fold (existing 1-minute cadence).
- Widget window starts hidden on launch.
- Left-click on the tray icon toggles the widget; right-click opens the menu
  (Show / Hide, Quit — unchanged).

## Implementation

All Rust-side; no frontend changes.

1. **`src-tauri/src/tray.rs`** — new module with pure
   `tray_title(&HashMap<String, UsageSnapshot>) -> Option<String>` doing the formatting above.
   Providers are ordered Claude, Codex regardless of map iteration order.
2. **`lib.rs`** — tray built with `TrayIconBuilder::with_id("main")`;
   `show_menu_on_left_click(false)`; `on_tray_icon_event` toggles the panel on
   left-click up. Remove the startup `panel.show()`.
3. **`poll.rs`** — after each snapshot fold, call `tray::update_tray_title`,
   which serializes writers behind a `TrayTitle` mutex, recomputes from the
   full snapshot map, skips unchanged titles, and clears with an empty string
   (`set_title(None)` is a no-op in tray-icon's macOS backend).

Rejected: rendering mini bar graphics into the tray bitmap (DPI/theme cost,
~10× the code, no informational gain).

## Error handling

- `tray_by_id` miss or `set_title` failure: log to stderr and retry next
  poll; the widget remains the fallback surface.
- Malformed/missing bars degrade per the omission rules above; never panic.

## Testing

Unit tests on `tray_title`: both providers, single provider, none (→ `None`),
missing weekly bar, unauthenticated skipped, clamping of out-of-range percents.
Tray wiring is asserted the same way existing `lib.rs` source-inclusion tests
work.
