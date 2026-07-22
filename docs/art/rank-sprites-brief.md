# Rank Sprite Sheets — Codex Art Brief

Generate per-rank mage sprite sheets for the Mana widget. These dress the two
provider familiars as the player climbs ranks. Match the existing chunky
pixel-art style — reference `public/sprites/claude-fire-poison.png` and
`public/sprites/codex-ice-lightning.png` before drawing anything.

## Sheet geometry contract (hard requirements)

- 448×336 RGBA PNG (2x retina; displayed at 224×168 CSS, each cell at 56px).
- 4 frame columns × 3 state rows of 112×112 cells.
- State rows top-to-bottom: **idle**, **working**, **hover**.
- ≥4px fully transparent margin inside every cell edge (alpha 0 — no
  anti-aliased spill into the margin).
- Per-cell visible pixels (alpha > 16) between 600 and 10,500 — costumes,
  halos, and wings must fit the budget and the cell.
- Per-row baseline spread ≤12px (the lowest visible pixel of the four frames
  in a row may not vary by more than 12px, so animation does not bounce).
- Transparent background everywhere outside the figure.
- Acceptance = `npm test` — `src/sprites.test.ts` decodes every
  `public/sprites/*-rank-*.png` and enforces all of the above automatically.

## File names (28 sheets)

`public/sprites/claude-rank-<tier>.png` and
`public/sprites/codex-rank-<tier>.png` for every tier:

`naked`, `plastic`, `wood`, `iron`, `bronze`, `silver`, `gold`, `platinum`,
`emerald`, `diamond`, `master`, `legend`, `champion`, `godlike`.

## The two mages

- **Claude mage**: cyan/blue palette (`#39ddff` → `#557cff`), fire/poison
  spell effects while working.
- **Codex mage**: magenta/pink palette (`#d75cff` → `#ff5ba8`), ice/lightning
  spell effects while working.

Keep each mage's silhouette, face, and palette recognizably the same
character across all 14 tiers — only the dress escalates.

## State rows

- **idle** (row 1): relaxed stance, subtle 4-frame bob/breathing loop.
- **working** (row 2): casting loop with the mage's elemental effects
  (fire/poison for Claude, ice/lightning for Codex) — effects count toward
  the pixel budget, keep them tight to the figure.
- **hover** (row 3): alert greeting/flourish loop (staff raise, spark).

## Tier dress progression

`naked` = simple robeless apprentice (plain undergarment, no gear). Each
tier adds armor/cosmetics of its material, escalating to full regalia:

| tier | dress |
|---|---|
| naked | robeless apprentice, bare hands, no ornament |
| plastic | flimsy gray-white plastic charm and toy staff topper |
| wood | wood-carved staff, wooden amulet, rope-belted robe |
| iron | iron pauldrons, riveted belt, iron-shod staff |
| bronze | bronze chestpiece, bronze staff cap, warm metal trim |
| silver | silver-trimmed robe, silver circlet, polished staff |
| gold | gilded robe, gold staff crown, gleaming clasps |
| platinum | pale platinum plates with fine filigree glow |
| emerald | emerald-studded regalia, green gem staff head |
| diamond | crystalline shoulder shards, prismatic accents |
| master | deep red war-mage mantle and burning sigils |
| legend | purple mythic robes with floating runes |
| champion | radiant gold-and-blue regalia, laurelled staff |
| godlike | halo, wings, streaming heavenly light (keep within cell + budget) |

## Generation

Run from the repo root; write files directly into `public/sprites/`. Then run
`npm test` — the atlas suite is the acceptance gate; iterate until green.
