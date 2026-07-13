# mana v0.3 - Party Roster UX redesign

2026-07-13 - approved by Dennis

Approved direction: Arcane sci-fi HUD with the Party Roster expanded layout.

## Goal

Make mana easier to read and more distinctly game-like without changing how it obtains usage data. The expanded widget must show every reset value without clipping, visually attach each familiar to its provider, expose the native macOS glass treatment, and label Codex limits by their real duration.

## Problems being fixed

- The expanded window remains 340 logical pixels wide, while row values use an 84px no-wrap column. Reset strings such as `Sun 12:51 PM` are clipped by the root's hidden overflow.
- The collapsed summary remains visible above the expanded detail, repeating information and leaving the mascots visually detached from the detailed provider sections.
- The CSS overlay is opaque enough to mute the native `HudWindow` vibrancy already applied by Tauri.
- Expanded headings and rows are only 9-11px and secondary text is too dim for quick scanning.
- Codex parsing assumes `primary_window` always means 5 hours and `secondary_window` always means weekly. Codex Pro can return a weekly-only limit as the primary window, producing a false `5h` label.

## Scope

### In scope

- Restyle the existing collapsed pill and expanded card as one coherent Arcane sci-fi HUD.
- Replace the expanded summary-plus-detail composition with the approved Party Roster layout.
- Reuse Clawd and Nimbus in provider-owned expanded sections and preserve their idle, working, and hover animation states.
- Fit the expanded window height to the rendered provider rows.
- Classify Codex windows from `limit_window_seconds` and test weekly-only Pro data.
- Improve typography, contrast, spacing, glass depth, and long-value handling.
- Retain stale, absent, low-mana, and reduced-motion behavior.

### Out of scope

- New usage endpoints, polling behavior, credential handling, or token refresh.
- New mascot art or animation states.
- Click-to-pin, settings, notifications, sounds, or a menu-bar mode.
- Framework migration or a new component library.

## Visual system

The UI remains a dark HUD, but the glass becomes visibly translucent instead of reading as a flat navy panel.

### Palette

- Void glass: `#080d1a`
- Frost text: `#edf3ff`
- Mist text: `#a8b5cc`
- Claude cyan: `#48d6ff`
- Arc indigo: `#6f7cff`
- Codex magenta: `#e45ab7`

Clawd's existing coral pixels provide a warm counterpoint without adding another UI accent.

### Type

- Body and values: the macOS system sans stack for legibility at utility-widget scale.
- Provider names and utility labels: the macOS monospace stack to evoke a game status console without turning all content into pixel text.
- Expanded provider names: 13px, bold.
- Row labels and reset metadata: 11px minimum.
- Percentages: 12px, semibold.
- Letter spacing remains zero except for short uppercase provider labels, where normal monospace glyph spacing is sufficient.

### Glass and HUD treatment

- Keep native `HudWindow` vibrancy as the physical glass layer.
- Lower the dark CSS tint so desktop color can show through while text contrast remains stable.
- Add one top-edge specular highlight, a thin cool border, and a faint 8px pixel grid that fades before the lower half.
- Keep segmented mana bars and provider glows. Do not add decorative badges, nested cards, or extra ornaments.
- The signature visual is the familiar-led Party Roster: each provider reads as a party member with its own mana reserves.

## Layout

### Collapsed state

- Remains `340x48` logical pixels.
- Retains two provider summaries separated by a quiet divider.
- Clawd remains adjacent to Claude; Nimbus remains adjacent to Codex.
- Percentage and reset values remain single-line and use the existing compact countdown format.
- The glass, border, typography, and segmented tracks adopt the new visual tokens.

### Expanded state

- Uses a 420px logical width.
- Hides the collapsed summary completely while expanded.
- Fits height to actual rendered content instead of reserving a fixed 248px panel.
- Stacks two unframed provider bands separated by one hairline divider.
- Each band uses a 44px familiar column and a flexible data column.
- Each provider header contains the provider name, plan, and a small activity signal.
- Each usage row uses `label | flexible track | max-content value` so the reset value cannot be truncated.
- The normal maximum of three Claude rows and two Codex rows must fit without overlap or scrollbars.

```text
+--------------------------------------------------+
| [Clawd]  CLAUDE  Max                            * |
|          5 hour  [###################] 96%  1h10m |
|          Weekly  [#######............] 36%  Tue...|
|          Fable   [...................]  0%  Tue...|
| ------------------------------------------------ |
| [Nimbus] CODEX  Pro                             * |
|          Weekly  [#########..........] 45%  Sun...|
+--------------------------------------------------+
```

The ellipses above represent abbreviated wireframe content only. The implemented values must render in full.

## Interaction and animation

- Hover still expands the widget; leaving collapses it after a 150ms delay so small pointer slips do not interrupt reading.
- Re-entering before the delay expires cancels collapse.
- Window resize operations remain serialized to avoid expansion/collapse races.
- Expanded height is remeasured when provider data changes while the card is open.
- Expansion keeps the current window origin unless the extra width would cross the active monitor's work area; in that case it shifts left only enough to keep the full HUD visible, then restores the collapsed origin on collapse.
- Every rendered sprite for a provider receives the same `idle`, `working`, or `hover` state.
- A single restrained scan highlight crosses a working provider's filled mana bars. It is disabled by `prefers-reduced-motion` along with existing sprite and low-mana animations.
- The existing low-mana warm pulse remains the only warning animation.

## Data behavior

### Codex window classification

`parse_codex` must classify each returned rate-limit window from `limit_window_seconds`, not the JSON field name:

| Duration | Semantic id | Display label |
|---:|---|---|
| `18000` seconds | `session` | `5 hour` |
| `604800` seconds | `weekly` | `Weekly` |
| missing or unknown | field-stable neutral id | `Primary` or `Secondary` |

- A Codex Pro response containing only a `primary_window` with `604800` seconds produces one `weekly` bar labeled `Weekly`.
- The current Pro Lite response with 18,000-second primary and 604,800-second secondary windows continues to produce `5 hour` followed by `Weekly`.
- Unknown durations must never be mislabeled as 5 hours or weekly.
- The collapsed summary continues to prefer a `session` bar when present and otherwise uses the first available bar, so weekly-only Pro still displays useful data.

Claude session labels are normalized from `5h` to `5 hour` for visual consistency; model-scoped labels such as `Fable` remain unchanged.

## Data states

- `ok`: full-color familiar, header, bars, percentages, and reset values.
- `stale`: preserve the full layout, reduce saturation, and show the existing fetched-age text without shifting rows.
- `absent`: preserve the provider-owned familiar and header, then show one readable login hint in place of rows.
- Empty or unknown reset timestamps omit only the reset suffix; percentages and tracks remain aligned.

## Implementation boundaries

- Keep Tauri 2, vanilla TypeScript, HTML, and CSS.
- Reuse `public/sprites/clawd.png` and `public/sprites/nimbus.png`; no generated assets are required.
- Add provider metadata to sprite elements and update all matching sprites together instead of relying on one element id per provider.
- Keep window geometry in explicit frontend constants; keep the initial Tauri window size synchronized with the collapsed dimensions.
- Do not touch Rust polling, credentials, endpoints, or 60-second cadence outside the Codex parser classification change.

## Verification

### Automated

- Add a captured or minimal representative Codex Pro weekly-only fixture.
- Add Rust parser tests for:
  - Pro Lite with 5-hour and weekly windows.
  - Pro with one weekly primary window.
  - Unknown duration receiving a neutral label.
- Update existing label expectations from `5h` to `5 hour` where normalization is applied.
- Run `npm test`.
- Run `npm run build`.
- Run `cargo fmt --check` and `cargo test` in `src-tauri`.

### Visual and runtime

- Verify the real Tauri window, not only a browser rendering, because macOS vibrancy is native.
- Capture and inspect collapsed and expanded states using live or representative data.
- Confirm the longest weekday reset values are fully visible at 420px.
- Confirm three Claude rows plus two Codex rows fit with no clipping, overlap, or excess blank panel.
- Confirm weekly-only Codex Pro reads `Weekly` in the expanded band.
- Confirm low-mana, stale, absent, working, hover, and reduced-motion states retain stable geometry.
- Verify expansion and delayed collapse through repeated pointer entry/exit and while the widget is positioned near a screen edge.

## Acceptance criteria

- No visible text is clipped or ellipsized in the normal expanded data set.
- The expanded view contains no duplicated collapsed summary.
- Each familiar is visually attached to its provider section.
- Codex labels reflect actual window duration, including weekly-only Pro.
- The native glass effect is visible while all text maintains practical contrast.
- The widget feels game-like through its roster hierarchy, segmented mana tracks, familiars, and restrained activity motion rather than decorative clutter.
- Existing polling, credential safety, position persistence, and activity detection continue to work.
