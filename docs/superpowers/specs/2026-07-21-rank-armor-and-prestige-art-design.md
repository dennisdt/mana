# Rank Armor, Mana Frames, and Prestige Art

Date: 2026-07-21
Status: approved direction; implementation pending

## Goal

Extend Mana's illustrated rank progression beyond the familiars. Every rank
must now dress the mana meters and the application shell in the same material
language, while prestige earns a distinct illustrated medallion. The result
should feel like one game HUD advancing from an undressed apprentice interface
to celestial regalia.

## Approved scope

1. Create 14 rank-specific mana-frame overlays, one for every tier from
   `naked` through `godlike`.
2. Preserve the original pre-iteration application border exactly; new armor
   shell experiments are deferred.
3. Move the Rank Up / Prestige control into the true top-right inset corner.
4. Remove the small blue and pink activity diamonds from provider headings.
5. Replace all 10 prestige badges with richly illustrated medallions.

## Visual language

The rank sprites remain the source of truth for each tier's material and
ornament. Meter and shell art follow the same progression:

| Tier | Mana frame and shell treatment |
|---|---|
| naked | dark forged channel, almost no ornament |
| plastic | molded pale-gray toy plates, simple seams |
| wood | carved timber rails, bindings, small knots |
| iron | riveted dark iron, heavy squared shoulders |
| bronze | warm bronze plates and restrained scrollwork |
| silver | polished silver, blue-white insets, clean filigree |
| gold | gilded rails, warm highlights, compact crown motif |
| platinum | pale platinum layering and fine luminous engraving |
| emerald | dark metal with emerald gems and green enamel |
| diamond | crystalline shoulders and prismatic facets |
| master | deep-crimson war plate with burning gold sigils |
| legend | royal-purple mythic plate with rune engraving |
| champion | radiant gold-and-blue tournament regalia |
| godlike | white-gold celestial armor, winged corners, halo geometry |

No tier uses floating star or diamond indicators. Gems may appear only when
physically mounted into armor or a medallion.

## Mana-frame assets

- Files: `public/hud/mana-bar-frame-<tier>.png`.
- Source size: 288×40 RGBA, displayed at 144×20 CSS pixels.
- The fill channel remains fixed at the existing geometry: 14px horizontal
  inset and 6px vertical inset at display size.
- The center channel must remain transparent/dark enough for cyan, magenta,
  and low-mana red fills to stay readable.
- Ornament cannot enter the channel or change fill measurement.
- Artwork has a fully transparent exterior and at least 2 source pixels of
  clear margin on every edge.
- The existing shared frame is the default CSS value when rank is missing or
  unreported. A known rank replaces that value with exactly one foreground
  frame, preventing doubled rails and end caps.
- Rank frames render above the live fill so mounted armor masks the channel
  ends cleanly; no fill may escape outside the visible frame opening.
- Motion remains CSS-only: the fill glint may animate, but the frame itself is
  static so usage is easy to read.

## Application perimeter

Restore and preserve the exact pre-iteration shell from the repository
baseline: the original rounded rank-tinted outline, subtle inset corner ticks,
top highlight, masked metallic ring, and existing Champion/Godlike glow motion.
Do not ship any of the experimental armor tokens, diagonal plates, beveled
corner guards, or extra rails from this iteration. Rank identity otherwise
remains in the familiar, mana frames, action tab, and prestige medallions.

The previous floating corner ticks and tiny provider activity diamonds are
removed. Working state is communicated by the existing animated mana glint
and a restrained increase in the familiar's elemental glow.

The XP fill gains a restrained moving highlight with two small circular
gem-glints. These are light reflections, not star-shaped icons, and stop under
reduced motion.

## Familiar framing correction

The Codex Champion atlas shown in the reported Pro-card screenshot has enough
cell margin, so the CSS viewport is not clipping it; the staff sunburst itself
has a flat, cut-looking authored silhouette. Regenerate that atlas with a
complete pointed sunburst and slightly more breathing room while preserving
the character, armor, 4×3 animation contract, and all three state rows.
Familiar containers explicitly keep ornamental overflow visible.

At the app's non-integer Retina scales, rank atlases use smooth native image
resampling instead of forced nearest-neighbor pixelation. Colored illumination
must not be applied as a filter to the rectangular sprite element because its
compositing bounds read as a clipped square. Render provider light as a larger
soft circular radial layer behind the familiar; retain at most a small neutral
grounding shadow on the sprite itself.

## Rank Up / Prestige control

- Position: absolute, `top: 4px; right: 4px` inside the application shell.
- Shape: compact angular armor tab, not a rounded pill.
- Material: inherits the active rank's frame colors and bevel.
- It stays above shell ornament, remains clickable without initiating window
  drag, and does not cover provider labels at supported widget sizes.
- Reduced-motion mode disables its pulse.

## Prestige medallions

- Files remain `public/badges/prestige-1.png` through
  `public/badges/prestige-10.png`.
- Each is 96×96 RGBA with a transparent exterior and displays at 24×24.
- All ten share one circular crest silhouette and dark-navy pixel outline so
  they read as a collection.
- Progression: forged silver seal, silver-gold crest, sapphire crown, ruby
  crown, emerald laurel, diamond aegis, purple runic crest, blue-gold champion
  seal, winged celestial medal, and a final white-gold cosmic crown.
- No text, numerals, generic five-point stars, loose sparkles, or scenery.
- Prestige counts above ten continue using the existing numeric overlay on
  the tenth medallion.

## Implementation boundaries

- Rank selection continues to come from `#root[data-rank]`.
- Sprite selection and progression math are unchanged.
- Provider fill hues remain cyan/blue for Claude and magenta/pink for Codex.
- Meter width calculations remain in `src/meter.ts` and do not change.
- Generated bitmap work is limited to `public/hud/` and `public/badges/`.
- Layout and armor integration are limited to `src/`, `index.html`, and tests.

## Acceptance

- All 14 mana overlays exist at exactly 288×40 RGBA.
- All 10 prestige medallions exist at exactly 96×96 RGBA.
- Assets have transparent exteriors, useful visible-pixel coverage, and no
  content touching their outer edge.
- Every rank resolves to its matching meter overlay.
- The shared meter frame remains the default when no known rank selects art.
- The activity-signal element and its blue/pink diamond styling are absent.
- Rank Up / Prestige is inset 4px from the top-right shell corner.
- The application perimeter matches the pre-iteration baseline with no new
  armor plates, diagonal guards, or additional rails.
- Existing fill geometry and percentage behavior remain unchanged.
- Reduced-motion behavior remains intact.
- The XP bar shows moving gem-like glints during normal motion and no sparkle
  animation under reduced motion.
- The Codex Champion staff head has a complete silhouette with no cut-looking
  edge, and its regenerated atlas still satisfies all sprite invariants.
- Familiar lighting fades outside the sprite cell without a square compositing
  edge, and the 2x atlases resample smoothly at non-integer app scale.
- `npm test` and `npm run build` pass.
- Final visual QA covers low, metallic, gem, elite, and Godlike tiers at the
  actual application scale.

## Out of scope

- Changes to XP, rank gates, prestige math, provider data, or window scaling.
- Animated bitmap meter frames.
- New sounds or rank-up ceremony behavior.
