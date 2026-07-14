# Slower Elemental Mage Animation Design

## Goal

Make the free-standing Claude and Codex mage loops easier to read by slowing every animation state to half of its current playback speed.

## Timing

The four-frame state rows retain their existing visual ordering and state behavior. Only their per-frame duration changes:

| State | Current frame duration | New frame duration | New loop duration |
| --- | ---: | ---: | ---: |
| Idle | 287.5 ms | 575 ms | 2.30 s |
| Working | 170 ms | 340 ms | 1.36 s |
| Hover | 205 ms | 410 ms | 1.64 s |

Working remains the quickest row, hover remains between working and idle, and all rows continue to use the same four atlas frames.

## Implementation

Update the centralized frame-duration map in `src/sprite-animation.ts`. The DOM frame scheduler, atlas background positions, state selection, and `prefers-reduced-motion` behavior remain unchanged.

## Verification

Update the focused frame-timing expectations first, verify they fail against the existing values, then update the timing map. Run the focused test suite and the complete frontend suite. Confirm the resulting source still freezes reduced-motion states on frame zero.
