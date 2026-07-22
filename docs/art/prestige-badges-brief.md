# Prestige Badges — Codex Art Brief

Generate the ten permanent prestige badges shown beside the level chip in
the Mana footer. Until these land, the widget renders a tinted CSS star in
each slot — the PNGs replace those fallbacks automatically.

## File contract (hard requirements)

- `public/badges/prestige-<n>.png`, n = 1–10.
- 96×96 RGBA PNG, fully transparent background.
- Displayed at 24×24 CSS — design **bold silhouettes that read at 24px**.
  No thin linework, no interior detail smaller than ~8px at full size.
- Same chunky pixel-art language as the sprite sheets in `public/sprites/`.

## Design progression

Escalating opulence across the ten badges
(laurel → shield → crown → wings → constellation → …):

| n | motif | tint family |
|---|---|---|
| 1 | laurel wreath | silver (`#dbe7f4`) |
| 2 | heater shield | silver |
| 3 | jeweled crown | silver |
| 4 | spread wings | gold (`#f2c968`) |
| 5 | constellation sigil | gold |
| 6 | rising phoenix | gold |
| 7 | cut diamond sigil | diamond (`#9be8ff`) |
| 8 | twin crystal dragons | diamond |
| 9 | celestial gate | diamond |
| 10 | radiant champion star | champion radiance (`#ffd75e` → `#3f8cff` gradient) |

The tint families mirror the CSS star fallbacks (silver → gold → diamond →
radiant), so a mixed shelf of PNGs and fallbacks still reads as one set.
Each badge should visibly outshine the previous one; badge 10 is the
permanent cap — beyond ten prestiges the UI overlays a count on it, so keep
its upper-right quadrant free of critical detail.

## Generation

Run from the repo root; write files directly into `public/badges/`
(create the directory). There is no automated acceptance gate for badges —
verify each reads cleanly when scaled to 24×24 — but `npm test` must remain
green (badges are not decoded by the sprite atlas suite).
