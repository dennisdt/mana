# Natural Motion Phases Design

## Goal

Keep the current relaxed MapleStory-inspired animation pacing while removing the unnatural impression that every character, mana glint, and activity pulse shares one clock.

## Root Cause

The exact-boundary scheduler correctly removes frame jitter, but it passes the same global `performance.now()` value to every sprite. Characters in the same state therefore select the same atlas frame at the same instant. The CSS mana glints and working-state pulses also use identical durations and start phases, so elements created together move together.

## Motion Direction

Use deterministic phase offsets rather than random startup values or different durations. Stable offsets keep the roster consistent across rerenders and launches, retain each animation's established personality, and remain straightforward to test.

## Sprite Phases

- Claude phase: `0` cycles.
- Codex phase: `0.375` cycles.
- A cycle remains four atlas frames.
- The offset is converted from a cycle fraction using the active state's duration, so the same visual separation applies in idle, working, and hover states.
- `spriteFrameAt` and `spriteFrameDelayAt` receive the provider phase and use the same phase-adjusted elapsed time. Frame selection and next-boundary scheduling must stay aligned.
- State changes preserve the global clock and provider phase; they do not restart either character at frame zero.
- Unknown providers fall back to the Claude phase of `0` cycles.

## CSS Phases

Provider cards define stable motion offsets:

- Claude: `0s`.
- Codex: `-0.8s`.

Mana rows add stable row offsets:

- Row 1: `0s`.
- Row 2: `-0.85s`.
- Row 3: `-1.7s`.
- Row 4: `-2.55s`.

The mana glint delay is the sum of the provider and row offsets. The working-state activity pulse uses only the provider offset. Existing animation durations, easing, colors, geometry, and fill-width transitions remain unchanged.

## Reduced Motion

`prefers-reduced-motion: reduce` continues to freeze every sprite on frame zero and disable mana glints and activity pulses. Phase offsets must not create timers when reduced motion is enabled.

## Scope

No sprite assets, mana-bar assets, layout, provider colors, polling, activity detection, window behavior, or release metadata change.

## Verification

- Add focused tests proving Claude and Codex select different frames at the same timestamp in every state.
- Prove phase-adjusted next-boundary delays match the displayed frame sequence.
- Verify unknown providers retain the zero-phase fallback.
- Verify the stylesheet declares provider and row offsets, combines them for glints, applies the provider offset to activity pulses, and preserves reduced-motion disabling.
- Run the complete frontend and Rust suites, production frontend build, native bundle build, and installed executable hash comparison.
