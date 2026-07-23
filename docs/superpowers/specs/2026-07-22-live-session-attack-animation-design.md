# Live Session Attack Animation Design

## Goal

Mana should play a provider's attack row only while that provider is actively
writing session activity after a prompt. Merely having Claude or Codex open,
or relaunching Mana while an idle session exists, must not trigger an attack.
The detached colored fragment visible to the left of Claude's Master attack
frames and the detached blue dots below Codex's Master attack frames must also
be removed without changing either character, aura, layout, or other rank
artwork.

## Confirmed Root Causes

The current backend equates an interactive CLI process existing with active
work. A Claude or Codex process can remain alive while it waits for the next
prompt, so process presence cannot distinguish thinking/token generation from
an idle session. `get_activity` also repeats that process check on every Mana
launch, which explains the immediate attack after a reload.

The Claude Master atlas contains separate opaque pixel islands in the far-left
strip of three working-row cells. The Codex Master atlas contains separate
blue/white pixel islands below the intended content in its first two working
frames. They are stored in the PNGs themselves, so allowing the sprites to
overflow correctly exposes them beside the intended characters.

## Selected Activity Signal

Use local session-log writes as the activity signal:

- Claude root: `~/.claude/projects/**/*.jsonl`
- Codex root: `~/.codex/sessions/**/*.jsonl`
- Poll interval: 1 second
- Active grace period: 2.5 seconds after the most recent observed write

At backend startup, recursively record each existing JSONL file's length and
modification time as a quiet baseline. Baseline discovery never counts as
activity. A later new file, length change, or modification-time change records
a write for that provider. The provider becomes active immediately and remains
active until 2.5 seconds pass without another observed change. Continuous
thinking, token streaming, and tool activity therefore keep the attack row
alive; quiet sessions return to idle naturally.

If Mana starts while a provider is already generating, the baseline is still
quiet, but the next write is detected within one poll and begins the attack.
Missing or unreadable session directories are treated as quiet and retried on
future polls. No session contents leave the Mac, and the watcher reads only
filesystem metadata.

## Backend Structure

`src-tauri/src/activity.rs` will separate pure tracking logic from Tauri:

1. A recursive JSONL fingerprint scan returns a map of path to file length and
   modification time.
2. A per-provider tracker owns the previous fingerprint map and optional last
   write time.
3. The first scan seeds the tracker without reporting activity. Later scans
   compare fingerprints, record writes, and calculate whether the 2.5-second
   grace period is still open.
4. A managed activity store holds the current `{ claude, codex }` state.
5. The watcher updates that store and emits the existing `activity` event only
   when the pair changes. The existing frontend state priority remains
   `hover/moving > working > idle`.
6. `get_activity` returns the managed state instead of rescanning processes,
   so a renderer reload receives the watcher's current answer without creating
   a false attack.

The old process-name matcher and process polling are removed. No new Rust
dependency is required.

## Master Atlas Cleanup

Clean only the detached far-left pixel islands in the working row of
`public/sprites/claude-rank-master.png`. Preserve the 448x336 RGBA atlas, its
4x3 grid, transparent cell margins, character silhouettes, skull/portal spell
effects, baselines, colors, and every other rank sheet. The visible left strip
of each cleaned Master working cell must be fully transparent; the intended
art begins farther inside each cell.

Clean only the detached bottom pixel islands in the first two working frames of
`public/sprites/codex-rank-master.png`. Preserve Codex's character, staff,
shield, portal, lightning, and the fourth working frame whose intended art
extends lower in its cell.

## Tests

Rust unit tests will prove:

- the initial fingerprint scan is quiet even when session files already exist;
- an existing file changing triggers activity;
- a new session file after baseline triggers activity;
- activity remains true inside the 2.5-second grace period;
- activity becomes false after the grace period expires;
- Claude and Codex trackers remain independent;
- missing directories remain quiet instead of failing the watcher.

The existing PNG decoder test will add Master-specific assertions that the
far-left strip of every Claude working-row cell and the detached bottom region
of the first three Codex working-row cells contain no visible alpha while the
normal atlas geometry and visibility checks continue to pass.

The full Rust and frontend test suites, production frontend build, and Tauri
release build must pass. The rebuilt app will then be launched locally for a
manual check: quiet on launch, attack within one second of live session writes,
idle roughly 2.5 seconds after writes stop, and no fragment beside Claude.

## Documentation and Scope

README activity wording will change from process presence to local session-log
write detection. This change does not alter rank progression, token totals,
usage polling, credentials, window sizing, sprite timing, hover behavior, or
any non-Master artwork.
