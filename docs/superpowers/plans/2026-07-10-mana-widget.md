# mana v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** macOS always-on-top Tauri widget showing depleting "mana" bars for Claude Code and Codex subscription usage (5h window, weekly, Fable weekly), per the approved spec at `docs/superpowers/specs/2026-07-10-mana-widget-design.md`.

**Architecture:** Rust backend polls two verified usage endpoints every 60s (credentials re-read fresh each tick, never refreshed), normalizes into `UsageSnapshot` events consumed by a vanilla-TS webview. The window is converted to a non-activating NSPanel (floating level, all Spaces, over fullscreen) with native vibrancy for the arcane-glass look.

**Tech Stack:** Tauri 2.11.x, Vite 6 + vanilla TypeScript, reqwest 0.12, time 0.3, tauri-nspanel (git, branch v2.1), window-vibrancy 0.7.1, tauri-plugin-window-state 2, vitest.

## Global Constraints

- **NEVER call any token-refresh flow, never read/write `refreshToken`/`refresh_token`, never write any credential file or Keychain item.** Read-only access, re-read fresh on every poll tick.
- Claude endpoint: `GET https://api.anthropic.com/api/oauth/usage` with headers `Authorization: Bearer <token>`, `anthropic-beta: oauth-2025-04-20`, `User-Agent: claude-code/<version>` (UA missing ⇒ permanent 429).
- Codex endpoint: `GET https://chatgpt.com/backend-api/wham/usage` with `Authorization: Bearer <tokens.access_token>`, `chatgpt-account-id: <tokens.account_id>`. Never use `/backend-api/codex/usage` (Cloudflare 403).
- Claude token source: macOS Keychain only — `security find-generic-password -s "Claude Code-credentials" -w` → `.claudeAiOauth.accessToken`. Never read `~/.claude/.credentials.json` (stale on this machine).
- Poll cadence: 60s per provider. A failed tick degrades to `stale` (keep last bars); it never blanks the UI.
- All Keychain access via `/usr/bin/security` CLI (stable Apple-signed binary ⇒ one-time "Always Allow"), never the keyring crate.
- tauri-nspanel is a git dependency: branch `v2.1`, **pin rev `a3122e894383aa068ec5365a42994e3ac94ba1b6`** (API on this branch: generic `to_panel::<P>()` + `tauri_panel!` macro, American-spelling `set_collection_behavior`).
- Mana bars display capacity REMAINING: fill width = `100 - used_percent`.
- Package identifier `com.vantasoft.mana`; Cargo package `mana`, lib `mana_lib`; Cargo edition stays "2021".

---

### Task 1: Scaffold Tauri app + icons

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/styles.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/.gitignore`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `scripts/gen-icon.py`, `app-icon.png` (generated), `src-tauri/icons/*` (generated)
- Modify: `.gitignore`

**Interfaces:**
- Produces: a running `npm run tauri dev` shell; `mana_lib::run()` entry; window label `"main"`; conf/capabilities that later tasks extend but do not restructure.

- [ ] **Step 1: Write frontend scaffold files**

`package.json`:

```json
{
  "name": "mana",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "vite": "^6.0.3",
    "typescript": "~5.6.2"
  }
}
```

`vite.config.ts`:

```ts
import { defineConfig } from "vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

`tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

`index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/styles.css" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>mana</title>
    <script type="module" src="/src/main.ts" defer></script>
  </head>
  <body>
    <div id="root">mana</div>
  </body>
</html>
```

`src/main.ts`:

```ts
console.log("mana webview up");
```

`src/styles.css`:

```css
html,
body {
  margin: 0;
  background: transparent;
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  user-select: none;
  cursor: default;
}
#root {
  height: 100vh;
  border-radius: 14px;
  background: rgba(13, 15, 32, 0.55);
  color: #dfe6ff;
  display: grid;
  place-items: center;
}
```

- [ ] **Step 2: Write src-tauri scaffold files**

`src-tauri/Cargo.toml`:

```toml
[package]
name = "mana"
version = "0.1.0"
description = "Gamer mana bars for Claude Code + Codex subscription usage"
authors = ["Dennis Tran"]
edition = "2021"

[lib]
name = "mana_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["macos-private-api", "tray-icon", "image-png"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

`src-tauri/.gitignore`:

```
/target/
/gen/schemas
```

`src-tauri/src/main.rs`:

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mana_lib::run()
}
```

`src-tauri/src/lib.rs`:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running mana");
}
```

`src-tauri/tauri.conf.json` (note `visible: true` for now — Task 7 flips it to `false` when `panel.show()` takes over):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "mana",
  "version": "0.1.0",
  "identifier": "com.vantasoft.mana",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "label": "main",
        "title": "mana",
        "width": 280,
        "height": 44,
        "transparent": true,
        "decorations": false,
        "shadow": false,
        "visible": true,
        "alwaysOnTop": true,
        "resizable": false,
        "acceptFirstMouse": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["app"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "main widget window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "core:window:allow-set-size"
  ]
}
```

Append to root `.gitignore`:

```
dist/
```

(`node_modules/` and `target/` are already there; `src-tauri/.gitignore` covers `gen/schemas`.)

- [ ] **Step 3: Generate the app icon set**

`scripts/gen-icon.py` (stdlib-only PNG writer; a cyan→violet mana crystal on transparency):

```python
import struct, zlib

S = 1024

def px(x, y):
    cx, cy = S / 2, S / 2
    d = abs(x - cx) + abs(y - cy)
    if d < 380:
        t = y / S
        return (
            int(56 + (168 - 56) * t),
            int(189 + (85 - 189) * t),
            int(248 + (247 - 248) * t),
            255,
        )
    if d < 402:
        return (223, 230, 255, 255)
    return (0, 0, 0, 0)

rows = b""
for y in range(S):
    rows += b"\x00" + bytes(v for x in range(S) for v in px(x, y))

def chunk(t, d):
    c = t + d
    return struct.pack(">I", len(d)) + c + struct.pack(">I", zlib.crc32(c))

png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(rows))
    + chunk(b"IEND", b""))
open("app-icon.png", "wb").write(png)
print("wrote app-icon.png")
```

Run:

```bash
python3 scripts/gen-icon.py
npm install
npm run tauri icon app-icon.png
```

Expected: `src-tauri/icons/` now contains `icon.icns`, `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.ico`, etc.

- [ ] **Step 4: Verify dev app runs**

```bash
npm run tauri dev
```

Expected: first compile takes minutes; then a small undecorated rounded dark rectangle reading "mana" floats on screen. Ctrl-C to stop.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 widget shell with generated icon set"
```

---

### Task 2: Usage parsers (Rust, TDD)

**Files:**
- Create: `src-tauri/tests/fixtures/claude_limits.json`, `src-tauri/tests/fixtures/claude_legacy.json`, `src-tauri/tests/fixtures/codex_wham.json`
- Create: `src-tauri/src/parsers.rs`
- Modify: `src-tauri/Cargo.toml` (add `time`), `src-tauri/src/lib.rs` (declare module)

**Interfaces:**
- Produces: `pub struct Bar { id: String, label: String, used_percent: f64, resets_at: Option<i64> }` (Serialize + Clone + Debug + PartialEq); `pub fn parse_claude(v: &serde_json::Value) -> Vec<Bar>`; `pub fn parse_codex(v: &serde_json::Value) -> (Vec<Bar>, Option<String>)`. Bar ids are `"session"`, `"weekly"`, `"model"`.

- [ ] **Step 1: Add the `time` dependency**

In `src-tauri/Cargo.toml` `[dependencies]` add:

```toml
time = { version = "0.3", features = ["parsing"] }
```

- [ ] **Step 2: Write fixtures (shapes captured live from both endpoints on 2026-07-10)**

`src-tauri/tests/fixtures/claude_limits.json`:

```json
{
  "five_hour": { "utilization": 26.0, "resets_at": "2026-07-10T19:39:59.928546+00:00" },
  "seven_day": { "utilization": 19.0, "resets_at": "2026-07-14T20:59:59+00:00" },
  "seven_day_opus": null,
  "seven_day_sonnet": null,
  "limits": [
    {
      "kind": "session",
      "group": "session",
      "percent": 26,
      "severity": "normal",
      "resets_at": "2026-07-10T19:39:59.928546+00:00",
      "is_active": false
    },
    {
      "kind": "weekly_all",
      "group": "weekly",
      "percent": 19,
      "severity": "normal",
      "resets_at": "2026-07-14T20:59:59+00:00",
      "is_active": false
    },
    {
      "kind": "weekly_scoped",
      "group": "weekly",
      "percent": 32,
      "severity": "normal",
      "resets_at": "2026-07-14T20:59:59+00:00",
      "scope": { "model": { "id": null, "display_name": "Fable" } },
      "is_active": true
    }
  ],
  "extra_usage": { "is_enabled": false },
  "spend": { "percent": 0, "enabled": false }
}
```

`src-tauri/tests/fixtures/claude_legacy.json`:

```json
{
  "five_hour": { "utilization": 26.0, "resets_at": "2026-07-10T19:39:59.928546+00:00" },
  "seven_day": { "utilization": 19.0, "resets_at": "2026-07-14T20:59:59+00:00" }
}
```

`src-tauri/tests/fixtures/codex_wham.json`:

```json
{
  "plan_type": "prolite",
  "rate_limit": {
    "allowed": true,
    "limit_reached": false,
    "primary_window": {
      "used_percent": 4,
      "limit_window_seconds": 18000,
      "reset_after_seconds": 17834,
      "reset_at": 1783727913
    },
    "secondary_window": {
      "used_percent": 1,
      "limit_window_seconds": 604800,
      "reset_after_seconds": 604634,
      "reset_at": 1784314713
    }
  },
  "additional_rate_limits": [],
  "credits": { "has_credits": false, "balance": "0" }
}
```

- [ ] **Step 3: Write failing tests**

`src-tauri/src/parsers.rs` (tests first — write the full file with `todo!()` bodies so the tests compile and fail):

```rust
use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Bar {
    pub id: String,
    pub label: String,
    pub used_percent: f64,
    pub resets_at: Option<i64>,
}

pub fn parse_claude(_v: &serde_json::Value) -> Vec<Bar> {
    todo!()
}

pub fn parse_codex(_v: &serde_json::Value) -> (Vec<Bar>, Option<String>) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(s: &str) -> serde_json::Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn claude_limits_array() {
        let bars = parse_claude(&load(include_str!("../tests/fixtures/claude_limits.json")));
        assert_eq!(bars.len(), 3);
        assert_eq!(bars[0], Bar {
            id: "session".into(),
            label: "5h".into(),
            used_percent: 26.0,
            resets_at: Some(1783712399),
        });
        assert_eq!(bars[1].id, "weekly");
        assert_eq!(bars[1].used_percent, 19.0);
        assert_eq!(bars[2], Bar {
            id: "model".into(),
            label: "Fable".into(),
            used_percent: 32.0,
            resets_at: Some(1784062799),
        });
    }

    #[test]
    fn claude_legacy_fallback() {
        let bars = parse_claude(&load(include_str!("../tests/fixtures/claude_legacy.json")));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].id, "session");
        assert_eq!(bars[0].used_percent, 26.0);
        assert_eq!(bars[0].resets_at, Some(1783712399));
        assert_eq!(bars[1].id, "weekly");
    }

    #[test]
    fn claude_garbage_yields_empty() {
        assert!(parse_claude(&load(r#"{"unexpected": true}"#)).is_empty());
        assert!(parse_claude(&load(r#"{"limits": [{"kind": "session"}]}"#)).is_empty());
    }

    #[test]
    fn codex_windows() {
        let (bars, plan) = parse_codex(&load(include_str!("../tests/fixtures/codex_wham.json")));
        assert_eq!(plan.as_deref(), Some("prolite"));
        assert_eq!(bars, vec![
            Bar { id: "session".into(), label: "5h".into(), used_percent: 4.0, resets_at: Some(1783727913) },
            Bar { id: "weekly".into(), label: "Weekly".into(), used_percent: 1.0, resets_at: Some(1784314713) },
        ]);
    }

    #[test]
    fn codex_garbage_yields_empty() {
        let (bars, plan) = parse_codex(&load(r#"{"detail": "unauthorized"}"#));
        assert!(bars.is_empty());
        assert!(plan.is_none());
    }
}
```

Declare the module in `src-tauri/src/lib.rs` — add as the first line:

```rust
pub mod parsers;
```

- [ ] **Step 4: Run tests to verify they fail**

```bash
cd src-tauri && cargo test
```

Expected: FAIL — panics with `not yet implemented` in all five tests.

- [ ] **Step 5: Implement the parsers**

Replace the two `todo!()` functions in `src-tauri/src/parsers.rs` with:

```rust
fn iso_to_epoch(s: &str) -> Option<i64> {
    time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
        .ok()
        .map(|t| t.unix_timestamp())
}

pub fn parse_claude(v: &serde_json::Value) -> Vec<Bar> {
    let mut bars = Vec::new();
    if let Some(limits) = v.get("limits").and_then(|l| l.as_array()) {
        for l in limits {
            let Some(percent) = l.get("percent").and_then(|p| p.as_f64()) else {
                continue;
            };
            let resets_at = l
                .get("resets_at")
                .and_then(|r| r.as_str())
                .and_then(iso_to_epoch);
            let (id, label) = match l.get("kind").and_then(|k| k.as_str()) {
                Some("session") => ("session", "5h".to_string()),
                Some("weekly_all") => ("weekly", "Weekly".to_string()),
                Some("weekly_scoped") => (
                    "model",
                    l.pointer("/scope/model/display_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Model")
                        .to_string(),
                ),
                _ => continue,
            };
            bars.push(Bar { id: id.into(), label, used_percent: percent, resets_at });
        }
    }
    if !bars.is_empty() {
        return bars;
    }
    for (key, id, label) in [("five_hour", "session", "5h"), ("seven_day", "weekly", "Weekly")] {
        if let Some(pct) = v.pointer(&format!("/{key}/utilization")).and_then(|u| u.as_f64()) {
            bars.push(Bar {
                id: id.into(),
                label: label.into(),
                used_percent: pct,
                resets_at: v
                    .pointer(&format!("/{key}/resets_at"))
                    .and_then(|r| r.as_str())
                    .and_then(iso_to_epoch),
            });
        }
    }
    bars
}

pub fn parse_codex(v: &serde_json::Value) -> (Vec<Bar>, Option<String>) {
    let plan = v.get("plan_type").and_then(|p| p.as_str()).map(String::from);
    let mut bars = Vec::new();
    for (key, id, label) in [
        ("primary_window", "session", "5h"),
        ("secondary_window", "weekly", "Weekly"),
    ] {
        if let Some(w) = v.pointer(&format!("/rate_limit/{key}")) {
            if let Some(pct) = w.get("used_percent").and_then(|p| p.as_f64()) {
                bars.push(Bar {
                    id: id.into(),
                    label: label.into(),
                    used_percent: pct,
                    resets_at: w.get("reset_at").and_then(|r| r.as_i64()),
                });
            }
        }
    }
    (bars, plan)
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cd src-tauri && cargo test
```

Expected: `test result: ok. 5 passed`

- [ ] **Step 7: Commit**

```bash
git add src-tauri
git commit -m "feat: parse Claude limits[] (+legacy) and Codex wham usage into Bars"
```

---

### Task 3: Credential readers (Rust, TDD where pure)

**Files:**
- Create: `src-tauri/src/creds.rs`
- Modify: `src-tauri/src/lib.rs` (declare module)

**Interfaces:**
- Produces: `pub struct CodexCreds { pub access_token: String, pub account_id: String }`; `pub fn codex_auth_path() -> PathBuf`; `pub fn read_codex_creds(path: &Path) -> Option<CodexCreds>`; `pub fn read_claude_token() -> Option<String>`; `pub fn claude_ua() -> String`; `pub fn parse_version(s: &str) -> Option<String>`.
- Safety: functions only READ. No function in this module may write files or call any refresh endpoint.

- [ ] **Step 1: Write the file with failing tests (pure parts stubbed with `todo!()`)**

`src-tauri/src/creds.rs`:

```rust
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CodexCreds {
    pub access_token: String,
    pub account_id: String,
}

/// $CODEX_HOME/auth.json, default ~/.codex/auth.json
pub fn codex_auth_path() -> PathBuf {
    std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".codex")
        })
        .join("auth.json")
}

pub fn read_codex_creds(_path: &Path) -> Option<CodexCreds> {
    todo!()
}

pub fn parse_version(_s: &str) -> Option<String> {
    todo!()
}

/// Read-only Keychain lookup of Claude Code's OAuth access token.
pub fn read_claude_token() -> Option<String> {
    let out = Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    Some(v.pointer("/claudeAiOauth/accessToken")?.as_str()?.to_string())
}

/// UA the Claude usage endpoint requires; real CLI version when available.
pub fn claude_ua() -> String {
    let version = Command::new("claude")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| parse_version(&String::from_utf8_lossy(&o.stdout)));
    format!("claude-code/{}", version.unwrap_or_else(|| "2.1.0".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_creds_from_auth_json() {
        let dir = std::env::temp_dir().join("mana-test-codex");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("auth.json");
        std::fs::write(
            &p,
            r#"{"openai_api_key": null, "tokens": {"id_token": "x.y.z", "access_token": "AT-123", "refresh_token": "RT-DO-NOT-TOUCH", "account_id": "acc-9"}, "last_refresh": "2026-07-02T21:46:07Z"}"#,
        )
        .unwrap();
        let c = read_codex_creds(&p).unwrap();
        assert_eq!(c.access_token, "AT-123");
        assert_eq!(c.account_id, "acc-9");
    }

    #[test]
    fn codex_creds_missing_file_or_fields() {
        assert!(read_codex_creds(Path::new("/nonexistent/auth.json")).is_none());
        let dir = std::env::temp_dir().join("mana-test-codex2");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("auth.json");
        std::fs::write(&p, r#"{"tokens": {}}"#).unwrap();
        assert!(read_codex_creds(&p).is_none());
    }

    #[test]
    fn version_from_cli_output() {
        assert_eq!(parse_version("2.1.34 (Claude Code)"), Some("2.1.34".into()));
        assert_eq!(parse_version("claude v2.2.0"), None);
        assert_eq!(parse_version("garbage"), None);
    }
}
```

Add to `src-tauri/src/lib.rs` next to the parsers declaration:

```rust
pub mod creds;
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test creds
```

Expected: FAIL — `not yet implemented` panics.

- [ ] **Step 3: Implement the two pure functions**

Replace the `todo!()` bodies:

```rust
pub fn read_codex_creds(path: &Path) -> Option<CodexCreds> {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    Some(CodexCreds {
        access_token: v.pointer("/tokens/access_token")?.as_str()?.to_string(),
        account_id: v.pointer("/tokens/account_id")?.as_str()?.to_string(),
    })
}

pub fn parse_version(s: &str) -> Option<String> {
    s.split_whitespace()
        .find(|t| {
            t.chars().next().is_some_and(|c| c.is_ascii_digit())
                && t.matches('.').count() >= 2
                && t.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .map(str::to_string)
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test creds
```

Expected: `3 passed`.

- [ ] **Step 5: Live smoke test of the Keychain read (values must not print)**

```bash
security find-generic-password -s "Claude Code-credentials" -w > /dev/null && echo "keychain read OK"
```

Expected: `keychain read OK` (may show a Keychain prompt once — click "Always Allow").

- [ ] **Step 6: Commit**

```bash
git add src-tauri
git commit -m "feat: read-only credential readers for Claude keychain and Codex auth.json"
```

---

### Task 4: Pollers, snapshot state, events, command

**Files:**
- Create: `src-tauri/src/poll.rs`
- Modify: `src-tauri/Cargo.toml` (add reqwest, tokio), `src-tauri/src/lib.rs` (wire state/setup/command)

**Interfaces:**
- Consumes: `parsers::{Bar, parse_claude, parse_codex}`, `creds::*` from Tasks 2–3.
- Produces: `pub struct UsageSnapshot { provider: String, bars: Vec<Bar>, plan: Option<String>, status: String, fetched_at: i64 }` (status ∈ `"ok" | "stale" | "absent"`); `pub type Snapshots = Mutex<HashMap<String, UsageSnapshot>>`; `pub fn fold_snapshot(...)`; `pub fn spawn_pollers(app: AppHandle)`; `#[tauri::command] pub fn get_snapshots(...) -> Vec<UsageSnapshot>`; event `"usage-update"` with a `UsageSnapshot` payload, emitted once per provider per 60s tick.

- [ ] **Step 1: Add dependencies**

In `src-tauri/Cargo.toml` `[dependencies]` add:

```toml
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["time"] }
```

- [ ] **Step 2: Write `poll.rs` with failing `fold_snapshot` tests**

`src-tauri/src/poll.rs`:

```rust
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

use crate::creds;
use crate::parsers::{self, Bar};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct UsageSnapshot {
    pub provider: String,
    pub bars: Vec<Bar>,
    pub plan: Option<String>,
    pub status: String,
    pub fetched_at: i64,
}

pub type Snapshots = Mutex<HashMap<String, UsageSnapshot>>;

/// A tick's fetch result: Some((bars, plan)) on success, None on any failure
/// (missing creds, HTTP error, unparseable body).
pub type FetchResult = Option<(Vec<Bar>, Option<String>)>;

pub fn fold_snapshot(
    _prev: Option<&UsageSnapshot>,
    _provider: &str,
    _result: FetchResult,
    _now: i64,
) -> UsageSnapshot {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar() -> Bar {
        Bar { id: "session".into(), label: "5h".into(), used_percent: 26.0, resets_at: Some(1783712399) }
    }

    #[test]
    fn success_yields_ok() {
        let s = fold_snapshot(None, "claude", Some((vec![bar()], None)), 100);
        assert_eq!(s.status, "ok");
        assert_eq!(s.provider, "claude");
        assert_eq!(s.fetched_at, 100);
        assert_eq!(s.bars.len(), 1);
    }

    #[test]
    fn failure_with_history_goes_stale_keeping_bars_and_time() {
        let prev = fold_snapshot(None, "codex", Some((vec![bar()], Some("prolite".into()))), 100);
        let s = fold_snapshot(Some(&prev), "codex", None, 160);
        assert_eq!(s.status, "stale");
        assert_eq!(s.bars, prev.bars);
        assert_eq!(s.plan.as_deref(), Some("prolite"));
        assert_eq!(s.fetched_at, 100); // age of the DATA, not of the attempt
    }

    #[test]
    fn failure_without_history_is_absent() {
        let s = fold_snapshot(None, "claude", None, 100);
        assert_eq!(s.status, "absent");
        assert!(s.bars.is_empty());
    }

    #[test]
    fn recovery_after_stale_is_ok_again() {
        let prev = fold_snapshot(None, "claude", Some((vec![bar()], None)), 100);
        let stale = fold_snapshot(Some(&prev), "claude", None, 160);
        let s = fold_snapshot(Some(&stale), "claude", Some((vec![bar()], None)), 220);
        assert_eq!(s.status, "ok");
        assert_eq!(s.fetched_at, 220);
    }
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod poll;
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd src-tauri && cargo test poll
```

Expected: FAIL — `not yet implemented`.

- [ ] **Step 4: Implement `fold_snapshot`, fetchers, poll loop, command**

Replace the `todo!()` and append below it in `src-tauri/src/poll.rs`:

```rust
pub fn fold_snapshot(
    prev: Option<&UsageSnapshot>,
    provider: &str,
    result: FetchResult,
    now: i64,
) -> UsageSnapshot {
    match (result, prev) {
        (Some((bars, plan)), _) => UsageSnapshot {
            provider: provider.into(),
            bars,
            plan,
            status: "ok".into(),
            fetched_at: now,
        },
        (None, Some(p)) => UsageSnapshot { status: "stale".into(), ..p.clone() },
        (None, None) => UsageSnapshot {
            provider: provider.into(),
            bars: Vec::new(),
            plan: None,
            status: "absent".into(),
            fetched_at: now,
        },
    }
}

async fn fetch_claude(client: &reqwest::Client, ua: &str) -> FetchResult {
    let token = creds::read_claude_token()?;
    let v: serde_json::Value = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", ua)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    let bars = parsers::parse_claude(&v);
    (!bars.is_empty()).then_some((bars, None))
}

async fn fetch_codex(client: &reqwest::Client) -> FetchResult {
    let c = creds::read_codex_creds(&creds::codex_auth_path())?;
    let v: serde_json::Value = client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(c.access_token)
        .header("chatgpt-account-id", c.account_id)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    let (bars, plan) = parsers::parse_codex(&v);
    (!bars.is_empty()).then_some((bars, plan))
}

fn epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn spawn_pollers(app: tauri::AppHandle) {
    for provider in ["claude", "codex"] {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let client = reqwest::Client::new();
            let ua = creds::claude_ua();
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                let result = match provider {
                    "claude" => fetch_claude(&client, &ua).await,
                    _ => fetch_codex(&client).await,
                };
                let next = {
                    let state = app.state::<Snapshots>();
                    let mut map = state.lock().unwrap();
                    let next = fold_snapshot(map.get(provider), provider, result, epoch_now());
                    map.insert(provider.to_string(), next.clone());
                    next
                };
                eprintln!("[mana] {} {} bars={}", provider, next.status, next.bars.len());
                let _ = app.emit("usage-update", &next);
            }
        });
    }
}

#[tauri::command]
pub fn get_snapshots(state: tauri::State<'_, Snapshots>) -> Vec<UsageSnapshot> {
    state.lock().unwrap().values().cloned().collect()
}
```

Note: a 401 self-heals without any refresh call — credentials are re-read fresh at the next 60s tick, picking up whatever token the CLI has since rotated. The tick that failed just shows `stale` for ≤60s.

Replace `src-tauri/src/lib.rs` entirely with:

```rust
pub mod creds;
pub mod parsers;
pub mod poll;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(poll::Snapshots::default())
        .setup(|app| {
            poll::spawn_pollers(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![poll::get_snapshots])
        .run(tauri::generate_context!())
        .expect("error while running mana");
}
```

(`app.handle()` needs no extra import; `Manager` is already imported inside `poll.rs`.)

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test
```

Expected: all parser + creds + poll tests pass (`12 passed` total).

- [ ] **Step 6: Live verification against both real accounts**

```bash
npm run tauri dev
```

Expected in terminal within ~5s of the window appearing:

```
[mana] claude ok bars=3
[mana] codex ok bars=2
```

(First Keychain access may prompt — "Always Allow".) Ctrl-C.

- [ ] **Step 7: Commit**

```bash
git add src-tauri
git commit -m "feat: 60s pollers with stale-degrading snapshots, usage-update events, get_snapshots command"
```

---

### Task 5: Frontend format helpers (TDD, vitest)

**Files:**
- Create: `src/format.ts`, `src/format.test.ts`
- Modify: `package.json` (vitest devDep + test script)

**Interfaces:**
- Produces: `manaLeft(usedPercent: number): number` (0–100 clamp of remaining); `fmtCountdown(resetsAt: number | null, nowMs: number): string` (`"2h 14m"`, `"14m"`, `"<1m"`, `"now"`, `""` when null); `fmtAge(fetchedAt: number, nowMs: number): string` (`"just now"`, `"3m ago"`).

- [ ] **Step 1: Add vitest**

In `package.json`: add `"test": "vitest run"` to scripts and `"vitest": "^3"` to devDependencies, then:

```bash
npm install
```

- [ ] **Step 2: Write failing tests**

`src/format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { fmtAge, fmtCountdown, manaLeft } from "./format";

describe("manaLeft", () => {
  it("inverts used into remaining", () => {
    expect(manaLeft(26)).toBe(74);
  });
  it("clamps", () => {
    expect(manaLeft(120)).toBe(0);
    expect(manaLeft(-5)).toBe(100);
  });
});

describe("fmtCountdown", () => {
  const now = 1_783_712_399_000; // ms
  it("hours and minutes", () => {
    expect(fmtCountdown(1_783_712_399 + 2 * 3600 + 14 * 60, now)).toBe("2h 14m");
  });
  it("minutes only", () => {
    expect(fmtCountdown(1_783_712_399 + 14 * 60, now)).toBe("14m");
  });
  it("under a minute", () => {
    expect(fmtCountdown(1_783_712_399 + 30, now)).toBe("<1m");
  });
  it("past reset", () => {
    expect(fmtCountdown(1_783_712_399 - 10, now)).toBe("now");
  });
  it("unknown", () => {
    expect(fmtCountdown(null, now)).toBe("");
  });
});

describe("fmtAge", () => {
  const now = 1_783_712_399_000;
  it("fresh", () => {
    expect(fmtAge(1_783_712_399 - 30, now)).toBe("just now");
  });
  it("minutes", () => {
    expect(fmtAge(1_783_712_399 - 190, now)).toBe("3m ago");
  });
});
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
npm test
```

Expected: FAIL — cannot resolve `./format`.

- [ ] **Step 4: Implement**

`src/format.ts`:

```ts
export function manaLeft(usedPercent: number): number {
  return Math.min(100, Math.max(0, 100 - usedPercent));
}

export function fmtCountdown(resetsAt: number | null, nowMs: number): string {
  if (resetsAt == null) return "";
  const s = Math.round(resetsAt - nowMs / 1000);
  if (s <= 0) return "now";
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return "<1m";
}

export function fmtAge(fetchedAt: number, nowMs: number): string {
  const m = Math.floor((nowMs / 1000 - fetchedAt) / 60);
  return m <= 0 ? "just now" : `${m}m ago`;
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
npm test
```

Expected: `9 passed`.

- [ ] **Step 6: Commit**

```bash
git add package.json package-lock.json src/format.ts src/format.test.ts
git commit -m "feat: mana/countdown/age formatting helpers with vitest"
```

---

### Task 6: Widget UI — pill, hover-expand card, arcane glass theme

**Files:**
- Modify: `index.html`, `src/styles.css`, `src/main.ts`

**Interfaces:**
- Consumes: event `"usage-update"` + command `get_snapshots` (Task 4 shapes), `format.ts` helpers (Task 5).
- Produces: final DOM/CSS contract: `body.expanded` toggles the card; window sizes `COLLAPSED 280×44` / `EXPANDED 300×248` (Task 7 keeps these).

- [ ] **Step 1: Replace `index.html` body**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/styles.css" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>mana</title>
    <script type="module" src="/src/main.ts" defer></script>
  </head>
  <body>
    <div id="root" data-tauri-drag-region>
      <div id="pill" data-tauri-drag-region>
        <div class="slot" id="pill-claude" data-tauri-drag-region></div>
        <div class="split" data-tauri-drag-region></div>
        <div class="slot" id="pill-codex" data-tauri-drag-region></div>
      </div>
      <div id="card" data-tauri-drag-region>
        <section id="card-claude" data-tauri-drag-region></section>
        <section id="card-codex" data-tauri-drag-region></section>
      </div>
    </div>
  </body>
</html>
```

- [ ] **Step 2: Replace `src/styles.css` with the arcane glass theme**

```css
:root {
  --ink: #dfe6ff;
  --ink-dim: rgba(223, 230, 255, 0.55);
  --claude-1: #38bdf8;
  --claude-2: #6366f1;
  --claude-glow: rgba(99, 102, 241, 0.65);
  --codex-1: #a855f7;
  --codex-2: #ec4899;
  --codex-glow: rgba(168, 85, 247, 0.65);
  --low-1: #fb7185;
  --low-2: #f43f5e;
  --low-glow: rgba(244, 63, 94, 0.7);
}

html,
body {
  margin: 0;
  height: 100%;
  background: transparent;
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  color: var(--ink);
  user-select: none;
  -webkit-user-select: none;
  cursor: default;
  overflow: hidden;
}

#root {
  height: 100vh;
  box-sizing: border-box;
  border-radius: 14px;
  background: rgba(13, 15, 32, 0.5);
  border: 1px solid rgba(122, 162, 255, 0.22);
  box-shadow: inset 0 0 24px rgba(90, 120, 255, 0.09);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* ---- collapsed pill ---- */
#pill {
  flex: 0 0 44px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 10px;
}
#pill .split {
  width: 1px;
  align-self: stretch;
  margin: 10px 0;
  background: rgba(122, 162, 255, 0.25);
}
.slot {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 10px;
  font-variant-numeric: tabular-nums;
}
.slot .gem {
  font-size: 11px;
  filter: drop-shadow(0 0 3px var(--glow));
}
.slot .nums {
  white-space: nowrap;
  color: var(--ink-dim);
}
.slot .nums b {
  color: var(--ink);
  font-weight: 600;
}

/* ---- mana bars ---- */
.track {
  flex: 1;
  min-width: 24px;
  height: 7px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.09);
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}
.fill {
  height: 100%;
  border-radius: inherit;
  width: 0;
  background: linear-gradient(90deg, var(--c1), var(--c2));
  box-shadow: 0 0 8px var(--glow);
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1);
}
.claude {
  --c1: var(--claude-1);
  --c2: var(--claude-2);
  --glow: var(--claude-glow);
}
.codex {
  --c1: var(--codex-1);
  --c2: var(--codex-2);
  --glow: var(--codex-glow);
}
.low {
  --c1: var(--low-1);
  --c2: var(--low-2);
  --glow: var(--low-glow);
}
.low .fill {
  animation: pulse 1.4s ease-in-out infinite;
}
@keyframes pulse {
  50% {
    opacity: 0.55;
    box-shadow: 0 0 14px var(--glow);
  }
}

/* ---- expanded card ---- */
#card {
  display: none;
  flex-direction: column;
  gap: 10px;
  padding: 2px 12px 12px;
}
body.expanded #card {
  display: flex;
}
#card section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.head {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
.head .plan,
.head .age {
  font-weight: 400;
  letter-spacing: normal;
  text-transform: none;
  font-size: 9px;
  color: var(--ink-dim);
}
.head .age {
  margin-left: auto;
}
.row {
  display: grid;
  grid-template-columns: 44px 1fr 84px;
  align-items: center;
  gap: 8px;
  font-size: 10px;
  font-variant-numeric: tabular-nums;
}
.row .lbl {
  color: var(--ink-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.row .val {
  text-align: right;
  white-space: nowrap;
  color: var(--ink-dim);
}
.row .val b {
  color: var(--ink);
  font-weight: 600;
}

/* ---- data states ---- */
.stale {
  filter: saturate(0.25) opacity(0.75);
}
.empty {
  font-size: 10px;
  color: var(--ink-dim);
}
```

- [ ] **Step 3: Replace `src/main.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { fmtAge, fmtCountdown, manaLeft } from "./format";

type Bar = {
  id: string;
  label: string;
  used_percent: number;
  resets_at: number | null;
};
type Snapshot = {
  provider: string;
  bars: Bar[];
  plan: string | null;
  status: string;
  fetched_at: number;
};

const COLLAPSED = new LogicalSize(280, 44);
const EXPANDED = new LogicalSize(300, 248);
const GEMS: Record<string, string> = { claude: "◆", codex: "●" };

const snapshots = new Map<string, Snapshot>();

function barHtml(s: Snapshot, bar: Bar): string {
  const left = manaLeft(bar.used_percent);
  const low = left < 30 ? " low" : "";
  return `<div class="track ${s.provider}${low}"><div class="fill" style="width:${left}%"></div></div>`;
}

function pillHtml(s: Snapshot | undefined, provider: string): string {
  if (!s || s.status === "absent" || s.bars.length === 0) {
    return `<span class="gem">${GEMS[provider]}</span><span class="nums">no data</span>`;
  }
  const session = s.bars.find((b) => b.id === "session") ?? s.bars[0];
  const left = manaLeft(session.used_percent);
  const cd = fmtCountdown(session.resets_at, Date.now());
  const stale = s.status === "stale" ? " stale" : "";
  return `<span class="gem ${s.provider}${stale}">${GEMS[provider]}</span>
    ${barHtml(s, session)}
    <span class="nums${stale}"><b>${Math.round(left)}%</b>${cd ? " · " + cd : ""}</span>`;
}

function cardHtml(s: Snapshot | undefined, provider: string): string {
  const name = provider === "claude" ? "Claude" : "Codex";
  if (!s || s.status === "absent" || s.bars.length === 0) {
    return `<div class="head">${name}</div><div class="empty">no data — log in via the ${provider} CLI</div>`;
  }
  const stale = s.status === "stale" ? " stale" : "";
  const age = s.status === "stale" ? `<span class="age">${fmtAge(s.fetched_at, Date.now())}</span>` : "";
  const plan = s.plan ? `<span class="plan">${s.plan}</span>` : "";
  const rows = s.bars
    .map((b) => {
      const left = manaLeft(b.used_percent);
      const cd = fmtCountdown(b.resets_at, Date.now());
      return `<div class="row${stale}">
        <span class="lbl">${b.label}</span>
        ${barHtml(s, b)}
        <span class="val"><b>${Math.round(left)}%</b>${cd ? " · " + cd : ""}</span>
      </div>`;
    })
    .join("");
  return `<div class="head">${name}${plan}${age}</div>${rows}`;
}

function render(): void {
  for (const provider of ["claude", "codex"]) {
    const s = snapshots.get(provider);
    document.getElementById(`pill-${provider}`)!.innerHTML = pillHtml(s, provider);
    document.getElementById(`card-${provider}`)!.innerHTML = cardHtml(s, provider);
  }
}

async function expand(): Promise<void> {
  await getCurrentWindow().setSize(EXPANDED);
  document.body.classList.add("expanded");
}

async function collapse(): Promise<void> {
  document.body.classList.remove("expanded");
  await getCurrentWindow().setSize(COLLAPSED);
}

document.body.addEventListener("mouseenter", () => void expand());
document.body.addEventListener("mouseleave", () => void collapse());

void listen<Snapshot>("usage-update", (e) => {
  snapshots.set(e.payload.provider, e.payload);
  render();
});

void invoke<Snapshot[]>("get_snapshots").then((all) => {
  for (const s of all) snapshots.set(s.provider, s);
  render();
});

setInterval(render, 1000);
```

- [ ] **Step 4: Type-check, test, and verify live**

```bash
npm test && npx tsc --noEmit
npm run tauri dev
```

Expected: pill shows two glowing bars with real percentages and countdowns within a few seconds; hovering expands the card showing Claude 5h/Weekly/Fable rows and Codex 5h/Weekly rows; moving the mouse away collapses it. Bars deplete left-to-right (74% mana at 26% used). Ctrl-C.

- [ ] **Step 5: Commit**

```bash
git add index.html src/styles.css src/main.ts
git commit -m "feat: arcane glass pill + hover-expand card rendering live usage"
```

---

### Task 7: Widgetization — NSPanel, vibrancy, tray, no Dock, position persistence

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: window label `"main"`, `poll::spawn_pollers`, `poll::get_snapshots`.
- Produces: the final `run()`; panel type `ManaPanel`.

- [ ] **Step 1: Add dependencies (nspanel pinned to rev)**

In `src-tauri/Cargo.toml` `[dependencies]` add:

```toml
tauri-plugin-window-state = "2"
tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", rev = "a3122e894383aa068ec5365a42994e3ac94ba1b6" }
window-vibrancy = "0.7.1"
```

- [ ] **Step 2: Replace `src-tauri/src/lib.rs`**

```rust
pub mod creds;
pub mod parsers;
pub mod poll;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};
use tauri_plugin_window_state::StateFlags;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

tauri_panel! {
    panel!(ManaPanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::POSITION)
                .build(),
        )
        .manage(poll::Snapshots::default())
        .setup(|app| {
            // Menu-bar-only app: no Dock icon, never activates as a regular app.
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_webview_window("main").unwrap();

            // Glass blur behind the webview. Active state is required: a
            // non-activating panel is never key, and FollowsWindowActiveState
            // would render the material permanently dim.
            apply_vibrancy(
                &window,
                NSVisualEffectMaterial::HudWindow,
                Some(NSVisualEffectState::Active),
                Some(14.0),
            )?;

            // Non-activating floating panel: hovers over every window and
            // fullscreen Space without ever stealing keyboard focus.
            let panel = window.to_panel::<ManaPanel>()?;
            panel.set_level(PanelLevel::Floating.value());
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
            panel.set_collection_behavior(
                CollectionBehavior::new()
                    .can_join_all_spaces()
                    .full_screen_auxiliary()
                    .stationary()
                    .into(),
            );
            panel.show(); // orderFrontRegardless — no activation

            let toggle = MenuItem::with_id(app, "toggle", "Show / Hide", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit mana", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => {
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
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            poll::spawn_pollers(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![poll::get_snapshots])
        .run(tauri::generate_context!())
        .expect("error while running mana");
}
```

- [ ] **Step 3: Flip the window to start hidden**

In `src-tauri/tauri.conf.json`, change `"visible": true` to `"visible": false` (the window-state plugin restores position while hidden; `panel.show()` in setup reveals it without a flash — POSITION-only flags deliberately exclude VISIBLE so the plugin never calls `set_focus`, which would activate the app).

- [ ] **Step 4: Verify the full widget behavior manually**

```bash
cd src-tauri && cargo test && cd .. && npm run tauri dev
```

Checklist (all must hold):
- Widget appears with real blur behind it (drag it over a bright window — glass effect visible).
- No Dock icon; tray icon (mana crystal) present in the menu bar.
- Clicking/hovering the widget never takes focus away from the app you were typing in (keep typing in a terminal while mousing over it).
- It stays visible over a full-screen app (make Safari or the IDE fullscreen; widget still floats).
- Dragging the pill moves the window; quit via tray → relaunch (`npm run tauri dev`) → it reappears at the dragged position.
- Tray "Show / Hide" toggles it; "Quit mana" exits.
- Hover-expand still works after panel conversion.

- [ ] **Step 5: Commit**

```bash
git add src-tauri
git commit -m "feat: non-activating vibrant NSPanel widget with tray, no Dock icon, position persistence"
```

---

### Task 8: Release build, install, README

**Files:**
- Modify: `src-tauri/Cargo.toml` (release profile)
- Create: `README.md`

**Interfaces:**
- Consumes: everything; this task ships it.

- [ ] **Step 1: Add size-optimized release profile**

Append to `src-tauri/Cargo.toml`:

```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = 3
panic = "abort"
strip = true
```

- [ ] **Step 2: Build and install**

```bash
npm run tauri build
cp -R src-tauri/target/release/bundle/macos/mana.app /Applications/
open /Applications/mana.app
```

Expected: build completes (warnings ok, no errors); the widget appears floating with live data. Check footprint: `ps aux | grep -i mana.app` — RSS well under 100MB (typically ~40–60MB for app + WKWebView).

- [ ] **Step 3: Write README.md**

```markdown
# mana

Gamer mana bars for your AI subscriptions. A tiny always-on-top macOS widget
showing how much Claude Code and Codex usage you have left — 5-hour window,
weekly, and Claude's model-scoped (Fable) weekly limit — as depleting mana
bars with reset countdowns. Hover to expand the full readout.

## How it reads usage

- **Claude Code**: `GET https://api.anthropic.com/api/oauth/usage` using the
  OAuth token Claude Code stores in the macOS Keychain
  (`Claude Code-credentials`), read via `/usr/bin/security`.
- **Codex**: `GET https://chatgpt.com/backend-api/wham/usage` using the token
  in `~/.codex/auth.json` (`$CODEX_HOME` respected).

Both are read-only: mana re-reads credentials fresh on every 60s poll and
**never refreshes or writes tokens**, so it cannot break your CLI logins.
If a token has expired (401), bars dim to a "stale" state until you next use
the CLI. Both endpoints are undocumented — expect occasional breakage.

## Build

    npm install
    npm run tauri build
    cp -R src-tauri/target/release/bundle/macos/mana.app /Applications/

First run: macOS Keychain will ask about `security` reading
"Claude Code-credentials" — choose **Always Allow**.

## Dev

    npm run tauri dev   # live widget
    npm test            # frontend unit tests
    cd src-tauri && cargo test   # Rust unit tests
```

- [ ] **Step 4: Final E2E pass (installed app)**

With `/Applications/mana.app` running: re-run the Task 7 checklist, then let it sit 5+ minutes and confirm bars/countdowns update (watch a countdown tick down; use Claude Code a bit and see the session bar deplete within a minute).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: release profile, README, shipped v0.1.0"
```
