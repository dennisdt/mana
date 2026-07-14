# Mana

Gamer mana bars for your AI subscriptions. This tiny always-on-top macOS widget
shows how much Claude Code and Codex usage you have left across session, weekly,
and model-scoped limits in a permanently expanded smoked-glass Party Roster.
Free-standing elemental mages sit beside their providers: Claude channels fire
and poison, while Codex casts ice and lightning. An original silver, gold, and
crystal fantasy frame holds equal-length live energy cores and complete reset
times. Codex limits are named from their actual duration, including weekly-only
Pro accounts.

## How it reads usage

- **Claude Code**: `GET https://api.anthropic.com/api/oauth/usage` using the
  OAuth token Claude Code stores in the macOS Keychain
  (`Claude Code-credentials`), read via `/usr/bin/security`.
- **Codex**: `GET https://chatgpt.com/backend-api/wham/usage` using the token
  in `~/.codex/auth.json` (`$CODEX_HOME` respected).

Both are read-only: Mana re-reads credentials fresh on every 60s poll and
**never refreshes or writes tokens**, so it cannot break your CLI logins.
If a token has expired (401), bars dim to a "stale" state until you next use
the CLI. Both endpoints are undocumented — expect occasional breakage.

## Familiars

Claude's fire/poison mage and Codex's ice/lightning mage are original illustrated
chibi characters generated as Retina sprite atlases. They breathe while idle,
cast while their CLI is actively running (local process check every 5s - nothing
leaves the machine), and celebrate whenever you hover or drag the widget. The
characters stand freely beside their provider bands without portrait frames.
Animations and the magical energy glint respect your Reduced Motion setting.

## Build

    npm install
    npm run tauri build
    cp -R src-tauri/target/release/bundle/macos/Mana.app /Applications/

First run: macOS Keychain will ask about `security` reading
"Claude Code-credentials" — choose **Always Allow**.

## Dev

    npm run tauri dev   # live widget
    npm test            # frontend unit tests
    cd src-tauri && cargo test   # Rust unit tests
