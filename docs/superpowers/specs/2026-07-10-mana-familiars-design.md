# mana v1.1 — Familiars (pet sprites) design

2026-07-10 · approved by Dennis (layout: companions flank the pill; characters: Clawd crab + cloud-bot (amended same day: user chose brand mascots for both); activity: local process check)

## What

Two original pixel-art companions living on the pill, modeled on the Codex desktop pet's behavior (floating overlay, draggable from anywhere, state-driven animation, persistent position — all already true of mana's window):

- **Clawd** (Claude) — coral pixel crab, homage to Anthropic's crab mascot, left end of the pill.
- **Nimbus** (Codex) — blue cloud-robot with a terminal visor, right end. Homage in original pixels; no OpenAI asset is copied.

## Animation states (priority: carried > hover > working > low > idle)

| State | Trigger | Clawd | Nimbus |
|---|---|---|---|
| idle | default | claw pinches, occasional blink | bob, visor blink |
| working | provider's CLI actively running | claws type furiously + spark | visor cursor types, antenna pings |
| hover | pointer over the widget | claws raised, little hop | excited leg kicks |
| carried | window is being moved (`onMoved` events; clears 300ms after last move) | claws tucked, legs wiggle | legs dangle |
| low | session mana < 30% | droopy eye stalks, dimmed shell | droopy/sleepy visor |

`prefers-reduced-motion: reduce` ⇒ static first frame of the active state.

## Art pipeline

- 16×16 px frames, 4 frames per state, two sprite sheets (`clawd.png`, `nimbus.png`, PNG-32 transparent) generated deterministically by `scripts/gen-sprites.py` (stdlib-only): hand-authored base pixel maps + programmatic state derivations (palette shifts, row shifts, particle overlays).
- Rendered at 2× (32×32) with `image-rendering: pixelated`; animated via CSS `steps(4)` over `background-position`.
- **Art checkpoint:** the generator also emits `sprites-preview.png` (8× grid of every frame); the user approves it before the sprites are wired into the UI.

## Activity detection (local, zero network)

Rust watcher task, 5s interval, emits `activity` event `{ claude: bool, codex: bool }` on change. Detection = provider CLI process present; exact match patterns verified empirically during implementation (the claude CLI runs under node — naive name matching could false-match Claude.app/other tools). If process matching proves ambiguous, fallback: newest session-file mtime (`~/.claude/projects/**/*.jsonl`, `~/.codex/sessions/**/*.jsonl`) within 60s = working. No credentials involved.

## Layout & polish

- Collapsed pill grows to 340×48 (sprite · bars · sprite); expanded card 340×248. Sprites carry `data-tauri-drag-region` like everything else — drag from anywhere, including the familiars.
- Gamer polish: mana tracks gain subtle RPG segment ticks (repeating-gradient), sprites cast a faint brand-colored glow on the glass.

## Out of scope

Codex-pet task tray/activity list (mana has no task queue), custom pet uploads, free-roaming desktop movement, sound.

## Ship

v0.2.0: version bump, rebuild, reinstall to /Applications, README updated with the familiars + activity watcher description.
