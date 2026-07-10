# mana — macOS usage-widget design

2026-07-10 · approved by Dennis (theme: arcane glass HUD; layout: slim pill + hover expand; architecture approved)

## What

A tiny always-on-top macOS widget ("mana") showing gamer-themed mana bars for **Claude Code** and **Codex** subscription usage. Mana = capacity remaining: bars start full and deplete as the 5-hour window fills. Shows per provider: 5h window remaining + reset countdown, weekly remaining, and (Claude only) the model-scoped "Fable" weekly bar.

## Data contracts (probe-verified 2026-07-10 on this machine)

### Claude Code

- `GET https://api.anthropic.com/api/oauth/usage`
- Headers: `Authorization: Bearer <accessToken>`, `anthropic-beta: oauth-2025-04-20`, `User-Agent: claude-code/<version>` (missing UA ⇒ persistent 429 with no Retry-After).
- Token: read-only, fresh **every poll**, from macOS Keychain: `security find-generic-password -s "Claude Code-credentials" -w` → JSON `.claudeAiOauth.accessToken` (`sk-ant-oat01-*`, ~8h TTL). `~/.claude/.credentials.json` is stale on this machine — never use it.
- Parse the canonical `limits[]` array; entries observed:
  - `{kind:"session", percent, resets_at(ISO-8601), severity, is_active}` → 5h bar
  - `{kind:"weekly_all", percent, resets_at, ...}` → weekly bar
  - `{kind:"weekly_scoped", percent, resets_at, scope:{model:{display_name:"Fable"}}, ...}` → Fable 5 bar
- Legacy fallback if `limits[]` absent: `five_hour.{utilization, resets_at}`, `seven_day.{utilization, resets_at}`. `seven_day_opus`/`seven_day_sonnet` are null — ignore.

### Codex

- `GET https://chatgpt.com/backend-api/wham/usage`
- Headers: `Authorization: Bearer <tokens.access_token>`, `chatgpt-account-id: <tokens.account_id>`; both read fresh every poll from `$CODEX_HOME/auth.json` (default `~/.codex/auth.json`).
- Parse: `rate_limit.primary_window.{used_percent, reset_after_seconds, reset_at(epoch s)}` (5h; `limit_window_seconds:18000`) and `rate_limit.secondary_window{...}` (weekly; 604800). `plan_type` from the endpoint (JWT claim lags). Ignore `additional_rate_limits[]`, `credits` in v1.
- `https://chatgpt.com/backend-api/codex/usage` returns Cloudflare 403 — never use. codex-cli has no usage subcommand.

### Token rules (safety-critical)

- **Never call either vendor's token-refresh flow.** Refresh tokens are single-use-rotate; a third-party refresh desyncs the CLI login (CodexBar's daily-logout bug).
- Never read or write `refreshToken`/`refresh_token` fields. Never write any credential file/Keychain item.
- On 401: re-read the credential once (CLI may have rotated it); if still 401 → `stale` UI state ("open claude/codex to re-auth"). Missing creds entirely → `absent` state.

## Architecture

Tauri v2 · Rust backend · vanilla TS/HTML/CSS frontend (no framework).

- **Poller (Rust)**: one tokio task per provider, 60s interval, immediate first tick. Each tick: read creds → GET → normalize → `app.emit("usage-update", snapshot)`. Keep last good snapshot; failures mark it stale rather than blanking bars.
- **Normalized snapshot** (serde, one shape for both providers):
  ```
  UsageSnapshot {
    provider: "claude" | "codex",
    bars: [ { id: "session"|"weekly"|"model", label, used_percent: f64, resets_at: Option<i64 epoch s> } ],
    plan: Option<String>,
    status: "ok" | "stale" | "absent",
    fetched_at: i64,
  }
  ```
- **Parsers**: pure functions `parse_claude(json) -> Vec<Bar>`, `parse_codex(json) -> Vec<Bar>`; defensive (unknown keys/nulls tolerated, legacy field fallback); unit-tested against fixture JSON captured from the live responses.
- **Frontend**: listens for `usage-update`, renders bars; a 1s local timer ticks the reset countdowns between polls. Mana shown = `100 - used_percent`.

## Window

- tauri.conf: `transparent:true, decorations:false, shadow:false, skipTaskbar:true, visible:false`, `app.macOSPrivateApi:true`; cargo features `["macos-private-api", "tray-icon"]`.
- `tauri-nspanel` (ahkohd, v2 branch, **pinned rev**): convert to non-activating panel — `nonactivating_panel` style mask, `PanelLevel::Floating`, collection behavior `can_join_all_spaces + full_screen_auxiliary + stationary`. Degraded fallback if the plugin ever breaks: plain `set_always_on_top(true)`.
- `set_activation_policy(ActivationPolicy::Accessory)` — no Dock icon. Tray icon menu: Show/Hide, Quit.
- Drag: `data-tauri-drag-region` on the pill + `core:window:allow-start-dragging`. Position persisted with `tauri-plugin-window-state` (window created hidden; plugin shows it → no flash).
- Rounded corners/shadow drawn in CSS (native shadow off on transparent windows).

## UI

Arcane glass HUD: dark translucent glass, cyan/blue Claude bar, violet Codex bar, soft glow.

- **Collapsed pill** (~230×36): two mana bars with % remaining + reset countdown each.
- **Hover ⇒ expanded card**: Claude 5h / weekly / Fable 5 rows, Codex 5h / weekly rows; each row = glowing bar, % remaining, countdown. Mouse-leave collapses. Window resizes to fit (panel stays non-activating throughout).
- **States**: mana < 30% → bar shifts warm (magenta/red) and pulses; `stale` → desaturated + age badge ("3m ago"); `absent` → dim row, "log in" hint.

## Testing

- `cargo test`: parser fixtures (real captured JSON incl. `limits[]`, wham shape, legacy shape, missing-fields), countdown/mana math.
- Manual E2E: run against live accounts; verify floats over full-screen apps, no focus steal, drag + position restore, stale state by renaming `auth.json` back and forth.

## Risks

| Risk | Mitigation |
|---|---|
| Undocumented endpoints drift/gray-zone | Read-only, 60s cadence, correct headers; defensive parsers with legacy fallback |
| Keychain prompt fatigue | `security` CLI (not keyring crate — ad-hoc rebuilds change signature); click "Always Allow" once |
| nspanel breaks on Tauri bump | Pin rev; alwaysOnTop degraded mode |
| Claude token expires when CLI idle (~8h) | `stale` state, never refresh |

## Out of scope (v1)

Codex sessions-JSONL fallback, burn-rate ETA, threshold notifications, menu-bar mini-mode, Windows/Linux, signing/notarization (personal ad-hoc build).
