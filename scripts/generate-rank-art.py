#!/usr/bin/env python3
"""Generate rank atlases and prestige badges as deterministic chunky pixel art."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
SPRITES = ROOT / "public" / "sprites"
BADGES = ROOT / "public" / "badges"

CELL = 56
ATLAS_COLUMNS = 4
ATLAS_ROWS = 3
SPRITE_SCALE = 2
BADGE_SIZE = 24
BADGE_SCALE = 4

TIERS = (
    "naked",
    "plastic",
    "wood",
    "iron",
    "bronze",
    "silver",
    "gold",
    "platinum",
    "emerald",
    "diamond",
    "master",
    "legend",
    "champion",
    "godlike",
)

OUTLINE = "#111525"
DEEP_OUTLINE = "#080A12"
SKIN = "#F3A45F"
SKIN_LIGHT = "#FFD08A"
SKIN_SHADOW = "#C96C3E"


@dataclass(frozen=True)
class ProviderPalette:
    primary: str
    secondary: str
    dark: str
    light: str
    eye: str
    effect_a: str
    effect_b: str
    effect_c: str


PROVIDERS = {
    "claude": ProviderPalette(
        primary="#39DDFF",
        secondary="#557CFF",
        dark="#173A70",
        light="#C2F7FF",
        eye="#2A1720",
        effect_a="#FF7A2D",
        effect_b="#FFE45E",
        effect_c="#68E86D",
    ),
    "codex": ProviderPalette(
        primary="#D75CFF",
        secondary="#FF5BA8",
        dark="#511B6C",
        light="#FFD0F0",
        eye="#8BF4FF",
        effect_a="#9BE8FF",
        effect_b="#EAFBFF",
        effect_c="#FFF06A",
    ),
}


MATERIALS = {
    "naked": ("#F5F0E8", "#C9BEAF", "#8C7E72"),
    "plastic": ("#E9EDF2", "#BBC5CF", "#7E8B99"),
    "wood": ("#9A5B32", "#D18A45", "#5A321E"),
    "iron": ("#6F7B89", "#AAB4BF", "#3D4652"),
    "bronze": ("#A95C32", "#E49A52", "#63341F"),
    "silver": ("#B9CADB", "#F3FAFF", "#6C7D90"),
    "gold": ("#D89A23", "#FFE26B", "#8B5916"),
    "platinum": ("#D5E2EC", "#FFFFFF", "#8096AC"),
    "emerald": ("#0A9B67", "#59F3A8", "#07563E"),
    "diamond": ("#56CFF1", "#D7FAFF", "#2D6FC0"),
    "master": ("#A52D37", "#FF6A4D", "#5D172B"),
    "legend": ("#7A3EC8", "#D376FF", "#3C1D75"),
    "champion": ("#E0A925", "#FFF18A", "#2E64C8"),
    "godlike": ("#FFF4B5", "#FFFFFF", "#65B9FF"),
}


class PixelCanvas:
    def __init__(self, size: tuple[int, int]):
        self.image = Image.new("RGBA", size, (0, 0, 0, 0))
        self.draw = ImageDraw.Draw(self.image)

    def rect(self, box: tuple[int, int, int, int], fill: str) -> None:
        self.draw.rectangle(box, fill=fill)

    def polygon(self, points: list[tuple[int, int]], fill: str) -> None:
        self.draw.polygon(points, fill=fill)

    def line(self, points: list[tuple[int, int]], fill: str, width: int = 1) -> None:
        self.draw.line(points, fill=fill, width=width, joint="curve")

    def ellipse(self, box: tuple[int, int, int, int], fill: str) -> None:
        self.draw.ellipse(box, fill=fill)


def outlined_rect(
    c: PixelCanvas,
    box: tuple[int, int, int, int],
    fill: str,
    outline: str = OUTLINE,
    border: int = 1,
) -> None:
    c.rect(box, outline)
    x0, y0, x1, y1 = box
    if x1 - x0 >= border * 2 and y1 - y0 >= border * 2:
        c.rect((x0 + border, y0 + border, x1 - border, y1 - border), fill)


def outlined_poly(
    c: PixelCanvas,
    points: list[tuple[int, int]],
    fill: str,
    outline: str = OUTLINE,
    width: int = 1,
) -> None:
    c.polygon(points, fill)
    closed = points + [points[0]]
    c.line(closed, outline, width)


def diamond(c: PixelCanvas, x: int, y: int, radius: int, fill: str, outline: str = OUTLINE) -> None:
    outlined_poly(c, [(x, y - radius), (x + radius, y), (x, y + radius), (x - radius, y)], fill, outline)


def sparkle(c: PixelCanvas, x: int, y: int, color: str, size: int = 2) -> None:
    c.rect((x, y - size, x, y + size), color)
    c.rect((x - size, y, x + size, y), color)
    c.rect((x, y, x, y), "#FFFFFF")


def draw_wings(c: PixelCanvas, y: int, material: tuple[str, str, str], frame: int) -> None:
    base, highlight, shadow = material
    flutter = (0, -1, 1, 0)[frame]
    left = [(24, y + 15), (18, y + 8), (10, y + 4 + flutter), (12, y + 11), (5, y + 9 + flutter),
            (9, y + 17), (4, y + 19), (13, y + 23), (21, y + 22)]
    right = [(32, y + 15), (38, y + 8), (46, y + 4 + flutter), (44, y + 11), (51, y + 9 + flutter),
             (47, y + 17), (52, y + 19), (43, y + 23), (35, y + 22)]
    outlined_poly(c, left, base, DEEP_OUTLINE)
    outlined_poly(c, right, base, DEEP_OUTLINE)
    c.line([(21, y + 18), (11, y + 10), (14, y + 18), (8, y + 18)], highlight, 2)
    c.line([(35, y + 18), (45, y + 10), (42, y + 18), (48, y + 18)], highlight, 2)
    c.rect((12, y + 21, 20, y + 23), shadow)
    c.rect((36, y + 21, 44, y + 23), shadow)


def draw_halo(c: PixelCanvas, x: int, y: int, frame: int) -> None:
    glow = ("#FFD75E", "#FFF2A3", "#FFFFFF", "#FFF2A3")[frame]
    c.line([(x - 8, y + 2), (x - 5, y), (x + 5, y), (x + 8, y + 2)], glow, 2)
    c.rect((x - 7, y + 3, x + 7, y + 3), "#D99928")
    sparkle(c, x - 10, y + 1, glow, 1)
    sparkle(c, x + 10, y + 1, glow, 1)


def draw_staff(
    c: PixelCanvas,
    provider: str,
    tier: str,
    state: int,
    frame: int,
    yoff: int,
) -> None:
    if tier == "naked":
        return

    rank = TIERS.index(tier)
    base, highlight, shadow = MATERIALS[tier]
    palette = PROVIDERS[provider]
    side = -1 if state != 2 else (1 if frame in (1, 2) else -1)
    x = 14 if side < 0 else 42
    top = 15 + yoff
    bottom = 49 + yoff
    if state == 1:
        x += (-1, 0, 1, 0)[frame]
        top -= (0, 2, 4, 2)[frame]
    elif state == 2:
        top -= (2, 5, 6, 3)[frame]

    shaft = "#8A562F" if rank >= 2 else "#D5D9DE"
    shaft_light = "#D59A58" if rank >= 2 else "#FFFFFF"
    c.line([(x + 1, top + 5), (x, bottom)], OUTLINE, 4)
    c.line([(x + 1, top + 5), (x, bottom)], shaft, 2)
    c.line([(x + 1, top + 8), (x + 1, bottom - 3)], shaft_light, 1)

    if tier == "plastic":
        c.ellipse((x - 4, top, x + 5, top + 8), OUTLINE)
        c.ellipse((x - 3, top + 1, x + 4, top + 7), base)
        c.rect((x - 1, top + 2, x + 2, top + 5), highlight)
        return

    if rank < 5:
        c.line([(x, top + 7), (x - 4, top + 3), (x - 2, top - 1)], OUTLINE, 2)
        c.line([(x, top + 7), (x - 3, top + 3), (x - 1, top)], base, 1)
        diamond(c, x + 2, top + 1, 3, highlight)
    elif rank < 8:
        c.ellipse((x - 5, top - 2, x + 5, top + 8), OUTLINE)
        c.ellipse((x - 4, top - 1, x + 4, top + 7), base)
        diamond(c, x, top + 3, 3, highlight)
    elif rank < 10:
        diamond(c, x, top + 2, 6, base, shadow)
        diamond(c, x, top + 2, 3, highlight, "#FFFFFF")
    elif rank == 10:
        c.polygon([(x, top - 4), (x + 3, top), (x + 6, top - 1), (x + 4, top + 5),
                   (x, top + 8), (x - 4, top + 5), (x - 6, top - 1), (x - 3, top)], OUTLINE)
        c.polygon([(x, top - 2), (x + 2, top + 1), (x + 4, top), (x + 3, top + 4),
                   (x, top + 6), (x - 3, top + 4), (x - 4, top), (x - 2, top + 1)], highlight)
    elif rank == 11:
        c.ellipse((x - 6, top - 3, x + 6, top + 9), shadow)
        c.ellipse((x - 4, top - 1, x + 4, top + 7), DEEP_OUTLINE)
        sparkle(c, x, top + 3, highlight, 2)
    elif rank == 12:
        c.polygon([(x, top - 4), (x + 3, top), (x + 7, top - 2), (x + 5, top + 4),
                   (x, top + 8), (x - 5, top + 4), (x - 7, top - 2), (x - 3, top)], OUTLINE)
        c.polygon([(x, top - 2), (x + 2, top + 2), (x + 5, top), (x + 3, top + 3),
                   (x, top + 6), (x - 3, top + 3), (x - 5, top), (x - 2, top + 2)], highlight)
        diamond(c, x, top + 2, 2, palette.secondary, "#FFFFFF")
    else:
        c.ellipse((x - 7, top - 4, x + 7, top + 10), "#FFD75E")
        c.ellipse((x - 5, top - 2, x + 5, top + 8), DEEP_OUTLINE)
        c.ellipse((x - 3, top, x + 3, top + 6), palette.primary)
        sparkle(c, x, top + 3, "#FFFFFF", 2)


def draw_claude_head(c: PixelCanvas, x: int, y: int, tier: str, frame: int) -> None:
    rank = TIERS.index(tier)
    # Familiar ears and crown-like forelock preserve the original Claude silhouette.
    outlined_poly(c, [(x - 12, y + 4), (x - 9, y - 5), (x - 4, y + 1)], SKIN_SHADOW)
    outlined_poly(c, [(x + 12, y + 4), (x + 9, y - 5), (x + 4, y + 1)], SKIN_SHADOW)
    c.polygon([(x - 10, y + 3), (x - 8, y - 2), (x - 5, y + 2)], SKIN_LIGHT)
    c.polygon([(x + 10, y + 3), (x + 8, y - 2), (x + 5, y + 2)], SKIN_LIGHT)
    outlined_poly(c, [(x - 11, y + 2), (x - 8, y - 1), (x - 5, y), (x - 3, y - 4),
                      (x, y), (x + 4, y - 5), (x + 6, y), (x + 10, y + 2),
                      (x + 11, y + 11), (x + 7, y + 17), (x, y + 20),
                      (x - 7, y + 17), (x - 11, y + 11)], SKIN)
    c.rect((x - 7, y + 3, x + 7, y + 5), SKIN_LIGHT)
    blink = frame == 2
    if blink:
        c.rect((x - 6, y + 10, x - 3, y + 10), OUTLINE)
        c.rect((x + 3, y + 10, x + 6, y + 10), OUTLINE)
    else:
        c.rect((x - 6, y + 9, x - 3, y + 12), OUTLINE)
        c.rect((x + 3, y + 9, x + 6, y + 12), OUTLINE)
        c.rect((x - 5, y + 9, x - 4, y + 10), "#FFFFFF")
        c.rect((x + 4, y + 9, x + 5, y + 10), "#FFFFFF")
    c.rect((x - 1, y + 14, x + 1, y + 15), SKIN_SHADOW)
    if rank >= 5:
        base, highlight, shadow = MATERIALS[tier]
        c.line([(x - 8, y + 1), (x - 4, y - 2), (x + 4, y - 2), (x + 8, y + 1)], shadow, 2)
        diamond(c, x, y - 2, 2 if rank < 8 else 3, highlight)
    if rank >= 12:
        c.polygon([(x - 10, y), (x - 5, y - 5), (x - 2, y - 1), (x, y - 7),
                   (x + 3, y - 1), (x + 7, y - 5), (x + 10, y)], "#E2AA28")
        c.rect((x - 7, y, x + 7, y + 2), "#FFF18A")


def draw_codex_head(c: PixelCanvas, x: int, y: int, tier: str, frame: int) -> None:
    palette = PROVIDERS["codex"]
    base, highlight, shadow = MATERIALS[tier]
    rank = TIERS.index(tier)
    hood = palette.primary if tier == "naked" else (base if rank in (10, 11, 12, 13) else palette.primary)
    hood_shadow = shadow if rank >= 10 else palette.dark
    outlined_poly(c, [(x - 11, y + 2), (x - 7, y - 4), (x + 5, y - 5), (x + 11, y + 2),
                      (x + 12, y + 13), (x + 7, y + 19), (x - 7, y + 19),
                      (x - 12, y + 13)], hood, DEEP_OUTLINE)
    c.polygon([(x - 9, y + 2), (x - 5, y - 2), (x + 5, y - 3), (x + 9, y + 3),
               (x + 8, y + 7), (x - 8, y + 7)], palette.light)
    c.rect((x - 10, y + 6, x + 10, y + 17), DEEP_OUTLINE)
    c.rect((x - 8, y + 7, x + 8, y + 15), "#081525")
    c.rect((x - 7, y + 8, x + 7, y + 9), "#10283E")
    if frame == 2:
        c.rect((x - 5, y + 11, x - 1, y + 11), palette.eye)
        c.rect((x + 3, y + 11, x + 5, y + 11), palette.eye)
    else:
        c.rect((x - 6, y + 10, x - 3, y + 12), palette.eye)
        c.rect((x + 3, y + 10, x + 6, y + 12), palette.eye)
        c.rect((x - 5, y + 10, x - 4, y + 10), "#FFFFFF")
        c.rect((x + 4, y + 10, x + 5, y + 10), "#FFFFFF")
    c.rect((x - 2, y + 14, x + 2, y + 14), palette.eye)
    if rank >= 5:
        c.line([(x - 7, y), (x - 3, y - 3), (x + 4, y - 3), (x + 8, y)], shadow, 2)
        diamond(c, x + 2, y - 3, 2 if rank < 8 else 3, highlight)
    if rank >= 12:
        c.polygon([(x - 9, y), (x - 5, y - 5), (x - 2, y - 1), (x + 1, y - 7),
                   (x + 4, y - 1), (x + 8, y - 5), (x + 10, y)], "#E2AA28")
        c.rect((x - 7, y, x + 8, y + 2), "#FFF18A")


def draw_spell_effects(c: PixelCanvas, provider: str, state: int, frame: int, yoff: int) -> None:
    if state == 0:
        return
    p = PROVIDERS[provider]
    if state == 1:
        phase = frame % 4
        if provider == "claude":
            # Alternating fire curl and poison motes around the casting hand.
            fire = [(41, 37 + yoff), (47, 33 + yoff), (49, 27 + yoff), (46, 22 + yoff),
                    (42, 25 + yoff), (45, 29 + yoff), (42, 32 + yoff)]
            c.line(fire, OUTLINE, 4)
            c.line(fire, p.effect_a, 2)
            c.rect((46, 25 + yoff, 48, 29 + yoff), p.effect_b)
            for dx, dy in ((-1, 0), (3, -5), (7, 1), (4, 6)):
                r = 1 + ((phase + dx) & 1)
                c.ellipse((39 + dx - r, 28 + dy + yoff - r, 39 + dx + r, 28 + dy + yoff + r), OUTLINE)
                c.ellipse((40 + dx - r, 28 + dy + yoff - r, 39 + dx + r, 27 + dy + yoff + r), p.effect_c)
        else:
            # Ice star at the hand, lightning zig-zag rising behind it.
            diamond(c, 44, 34 + yoff, 5 + (phase & 1), p.effect_a, DEEP_OUTLINE)
            c.rect((43, 29 + yoff, 45, 39 + yoff), p.effect_b)
            c.rect((39, 33 + yoff, 49, 35 + yoff), p.effect_b)
            bolt = [(47, 29 + yoff), (51, 24 + yoff), (48, 23 + yoff), (52, 17 + yoff),
                    (47, 20 + yoff), (49, 14 + yoff)]
            c.line(bolt, OUTLINE, 4)
            c.line(bolt, p.effect_c, 2)
        sparkle(c, 39 + phase, 18 + yoff + phase, p.effect_b, 1)
    else:
        # Greeting row keeps the flourish smaller than the working spell.
        side = 42 if frame in (0, 3) else 14
        sparkle(c, side, 18 + yoff - (frame & 1), p.effect_a, 2)
        sparkle(c, 47 if side > 28 else 9, 27 + yoff, p.effect_b, 1)


def draw_body(
    c: PixelCanvas,
    provider: str,
    tier: str,
    state: int,
    frame: int,
    xoff: int,
    yoff: int,
) -> None:
    p = PROVIDERS[provider]
    rank = TIERS.index(tier)
    base, highlight, shadow = MATERIALS[tier]
    cx = 28 + xoff
    shoulder_y = 33 + yoff
    waist_y = 42 + yoff
    baseline = 52 + yoff

    if tier == "godlike":
        draw_wings(c, 13 + yoff, MATERIALS[tier], frame)

    # Mantles and cloaks create large silhouette steps from master onward.
    if rank >= 2:
        robe_shadow = p.dark if rank < 10 else shadow
        outlined_poly(c, [(cx - 9, shoulder_y + 2), (cx - 13, baseline - 2), (cx - 5, baseline),
                          (cx, baseline - 3), (cx + 5, baseline), (cx + 13, baseline - 2),
                          (cx + 9, shoulder_y + 2)], robe_shadow)
        c.polygon([(cx - 6, shoulder_y + 4), (cx - 8, baseline - 4), (cx - 2, baseline - 2),
                   (cx, shoulder_y + 6), (cx + 2, baseline - 2), (cx + 8, baseline - 4),
                   (cx + 6, shoulder_y + 4)], p.primary)
        c.rect((cx - 1, shoulder_y + 6, cx + 1, baseline - 3), p.light)
        if rank >= 6:
            c.rect((cx - 8, baseline - 6, cx + 8, baseline - 4), base)
        if rank >= 10:
            c.polygon([(cx - 12, shoulder_y - 1), (cx - 17, baseline - 5), (cx - 11, baseline - 2),
                       (cx - 7, shoulder_y + 4)], base)
            c.polygon([(cx + 12, shoulder_y - 1), (cx + 17, baseline - 5), (cx + 11, baseline - 2),
                       (cx + 7, shoulder_y + 4)], base)
            c.line([(cx - 15, baseline - 5), (cx - 10, shoulder_y + 2)], highlight, 2)
            c.line([(cx + 15, baseline - 5), (cx + 10, shoulder_y + 2)], highlight, 2)
    else:
        # Robeless apprentice: plain tunic, shorts, bare hands, no ornament.
        outlined_rect(c, (cx - 8, shoulder_y, cx + 8, waist_y + 3), p.secondary)
        c.rect((cx - 5, shoulder_y + 2, cx + 5, waist_y), p.primary)
        c.rect((cx - 7, waist_y + 2, cx - 1, baseline - 3), "#F5F0E8")
        c.rect((cx + 1, waist_y + 2, cx + 7, baseline - 3), "#F5F0E8")

    # Boots / feet lock every row to the same visual baseline family.
    c.rect((cx - 9, baseline - 4, cx - 1, baseline), OUTLINE)
    c.rect((cx + 1, baseline - 4, cx + 9, baseline), OUTLINE)
    c.rect((cx - 7, baseline - 3, cx - 1, baseline - 1), shadow if rank else SKIN_SHADOW)
    c.rect((cx + 1, baseline - 3, cx + 7, baseline - 1), shadow if rank else SKIN_SHADOW)

    # Torso and material chest treatment.
    torso_color = p.primary if rank < 4 else base
    outlined_poly(c, [(cx - 8, shoulder_y), (cx + 8, shoulder_y), (cx + 9, waist_y),
                      (cx, waist_y + 3), (cx - 9, waist_y)], torso_color)
    c.polygon([(cx - 5, shoulder_y + 2), (cx, shoulder_y + 4), (cx + 5, shoulder_y + 2),
               (cx + 6, waist_y - 1), (cx, waist_y + 1), (cx - 6, waist_y - 1)],
              p.secondary if rank < 4 else highlight)

    if tier == "plastic":
        c.rect((cx - 5, shoulder_y + 3, cx + 5, shoulder_y + 5), highlight)
        diamond(c, cx, waist_y - 2, 2, base)
    elif tier == "wood":
        c.line([(cx - 6, shoulder_y + 3), (cx + 6, waist_y - 1)], "#E1B56E", 2)
        c.rect((cx - 8, waist_y - 1, cx + 8, waist_y + 1), "#8A562F")
        diamond(c, cx, waist_y, 2, base)
    elif rank >= 3:
        c.rect((cx - 8, waist_y - 1, cx + 8, waist_y + 1), shadow)
        c.rect((cx - 5, waist_y, cx + 5, waist_y), highlight)
        if rank >= 4:
            diamond(c, cx, shoulder_y + 6, 3, highlight)

    # Arms respond visibly to state.
    if state == 0:
        left_hand = (cx - 11, waist_y - 2)
        right_hand = (cx + 11, waist_y - 2)
    elif state == 1:
        left_hand = (cx - 13, waist_y - 6 + (frame & 1))
        right_hand = (cx + 15, waist_y - 9 - (frame & 1))
    else:
        left_hand = (cx - 12, waist_y - 5)
        right_hand = (cx + 14, shoulder_y - (frame % 3))

    for hand_x, hand_y, side in ((*left_hand, -1), (*right_hand, 1)):
        sx = cx + side * 7
        sy = shoulder_y + 3
        c.line([(sx, sy), (hand_x, hand_y)], OUTLINE, 5)
        c.line([(sx, sy), (hand_x, hand_y)], p.secondary if rank < 3 else base, 3)
        c.rect((hand_x - 2, hand_y - 2, hand_x + 2, hand_y + 2), OUTLINE)
        c.rect((hand_x - 1, hand_y - 1, hand_x + 1, hand_y + 1),
               SKIN if provider == "claude" else p.light)

    # Armor silhouette progression: pauldrons, chest plates, gems, shards, runes.
    if rank >= 3:
        outlined_poly(c, [(cx - 8, shoulder_y), (cx - 13, shoulder_y + 1),
                          (cx - 12, shoulder_y + 5), (cx - 7, shoulder_y + 4)], base)
        outlined_poly(c, [(cx + 8, shoulder_y), (cx + 13, shoulder_y + 1),
                          (cx + 12, shoulder_y + 5), (cx + 7, shoulder_y + 4)], base)
        c.rect((cx - 11, shoulder_y + 1, cx - 8, shoulder_y + 2), highlight)
        c.rect((cx + 8, shoulder_y + 1, cx + 11, shoulder_y + 2), highlight)
    if rank >= 7:
        c.line([(cx - 7, shoulder_y + 5), (cx, waist_y), (cx + 7, shoulder_y + 5)], highlight, 2)
    if rank == 8:
        for gx, gy in ((cx, shoulder_y + 5), (cx - 8, waist_y - 2), (cx + 8, waist_y - 2)):
            diamond(c, gx, gy, 2, "#59F3A8", "#D8FFE9")
    if rank == 9:
        outlined_poly(c, [(cx - 11, shoulder_y + 1), (cx - 8, shoulder_y - 6),
                          (cx - 4, shoulder_y + 2)], highlight, shadow)
        outlined_poly(c, [(cx + 11, shoulder_y + 1), (cx + 8, shoulder_y - 6),
                          (cx + 4, shoulder_y + 2)], highlight, shadow)
        sparkle(c, cx, waist_y - 2, "#FFFFFF", 2)
    if rank == 10:
        c.rect((cx - 6, shoulder_y + 7, cx - 4, shoulder_y + 9), "#FFCC4C")
        c.rect((cx + 4, shoulder_y + 7, cx + 6, shoulder_y + 9), "#FFCC4C")
    if rank == 11:
        for rx, ry in ((cx - 13, shoulder_y + 7), (cx + 13, shoulder_y + 9), (cx, waist_y + 5)):
            diamond(c, rx, ry, 2, "#E4A0FF", "#FFFFFF")
    if rank >= 12:
        c.line([(cx - 7, shoulder_y + 2), (cx, waist_y - 1), (cx + 7, shoulder_y + 2)], "#FFF18A", 2)
        diamond(c, cx, waist_y - 2, 2, p.secondary, "#FFFFFF")


def draw_sprite_frame(provider: str, tier: str, state: int, frame: int) -> Image.Image:
    c = PixelCanvas((CELL, CELL))
    bob_sets = (
        (0, -1, 0, 1),
        (0, -1, -1, 0),
        (0, 0, -1, 0),
    )
    x_sets = (
        (-1, 0, 1, 0),
        (0, 0, 1, 0),
        (0, 1, 0, -1),
    )
    yoff = bob_sets[state][frame]
    xoff = x_sets[state][frame]
    rank = TIERS.index(tier)

    if tier == "godlike":
        draw_halo(c, 28 + xoff, 4 + yoff, frame)

    draw_spell_effects(c, provider, state, frame, yoff)
    draw_staff(c, provider, tier, state, frame, yoff)
    draw_body(c, provider, tier, state, frame, xoff, yoff)

    head_y = 14 + yoff
    if provider == "claude":
        draw_claude_head(c, 28 + xoff, head_y, tier, frame)
    else:
        draw_codex_head(c, 28 + xoff, head_y, tier, frame)

    # High-tier ambient magic adds motion without dominating the character.
    if rank >= 11:
        ambient = MATERIALS[tier][1]
        sparkle(c, 7 + frame * 2, 13 + (frame % 2) * 4, ambient, 1)
        sparkle(c, 49 - frame * 2, 20 - (frame % 2) * 3, ambient, 1)
    return c.image


def generate_atlas(provider: str, tier: str) -> Image.Image:
    atlas = Image.new("RGBA", (CELL * ATLAS_COLUMNS, CELL * ATLAS_ROWS), (0, 0, 0, 0))
    for state in range(ATLAS_ROWS):
        for frame in range(ATLAS_COLUMNS):
            atlas.alpha_composite(draw_sprite_frame(provider, tier, state, frame), (frame * CELL, state * CELL))
    return atlas.resize((448, 336), Image.Resampling.NEAREST)


def badge_canvas() -> PixelCanvas:
    return PixelCanvas((BADGE_SIZE, BADGE_SIZE))


def badge_laurel(c: PixelCanvas, color: str, bright: str) -> None:
    for x, mirror in ((7, -1), (16, 1)):
        c.line([(x, 19), (x + mirror * 3, 15), (x + mirror * 4, 9)], OUTLINE, 4)
        c.line([(x, 19), (x + mirror * 3, 15), (x + mirror * 4, 9)], color, 2)
        for y, dx in ((16, 1), (12, 2), (8, 3)):
            c.ellipse((x + mirror * dx - 2, y - 2, x + mirror * dx + 1, y + 1), OUTLINE)
            c.rect((x + mirror * dx - 1, y - 1, x + mirror * dx, y), bright)
    c.rect((8, 19, 15, 21), OUTLINE)
    c.rect((9, 19, 14, 20), bright)


def badge_shield(c: PixelCanvas, color: str, bright: str, shadow: str) -> None:
    outlined_poly(c, [(5, 4), (19, 4), (20, 12), (17, 18), (12, 22), (7, 18), (4, 12)], color, DEEP_OUTLINE, 2)
    c.polygon([(7, 6), (12, 7), (12, 19), (8, 16), (6, 11)], bright)
    c.polygon([(12, 7), (17, 6), (18, 12), (15, 17), (12, 19)], shadow)
    c.rect((10, 8, 13, 16), bright)
    c.rect((8, 11, 16, 14), bright)


def badge_crown(c: PixelCanvas, color: str, bright: str, jewel: str) -> None:
    outlined_poly(c, [(3, 7), (7, 11), (10, 4), (13, 11), (18, 5), (21, 8),
                      (19, 18), (5, 18)], color, DEEP_OUTLINE, 2)
    c.rect((6, 14, 18, 17), bright)
    diamond(c, 12, 13, 3, jewel, "#FFFFFF")


def badge_wings(c: PixelCanvas, color: str, bright: str) -> None:
    left = [(11, 18), (6, 19), (2, 16), (6, 15), (1, 11), (7, 12), (3, 6), (10, 11)]
    right = [(13, 18), (18, 19), (22, 16), (18, 15), (23, 11), (17, 12), (21, 6), (14, 11)]
    outlined_poly(c, left, color, DEEP_OUTLINE, 2)
    outlined_poly(c, right, color, DEEP_OUTLINE, 2)
    c.line([(9, 15), (4, 11)], bright, 2)
    c.line([(15, 15), (20, 11)], bright, 2)
    diamond(c, 12, 16, 3, bright)


def badge_constellation(c: PixelCanvas, color: str, bright: str) -> None:
    nodes = [(5, 16), (8, 7), (13, 12), (18, 5), (20, 17), (12, 20)]
    c.line(nodes + [nodes[0]], OUTLINE, 5)
    c.line(nodes + [nodes[0]], color, 3)
    for x, y in nodes:
        c.rect((x - 2, y - 2, x + 2, y + 2), OUTLINE)
        c.rect((x - 1, y - 1, x + 1, y + 1), bright)


def badge_phoenix(c: PixelCanvas, color: str, bright: str, shadow: str) -> None:
    outlined_poly(c, [(12, 3), (15, 8), (21, 6), (18, 12), (22, 14), (16, 16),
                      (15, 21), (12, 17), (9, 22), (8, 16), (2, 14), (6, 11),
                      (3, 6), (10, 9)], color, DEEP_OUTLINE, 2)
    c.polygon([(12, 5), (14, 11), (12, 18), (10, 11)], bright)
    c.rect((11, 8, 14, 11), shadow)
    c.rect((13, 7, 16, 8), bright)


def badge_diamond(c: PixelCanvas, color: str, bright: str, shadow: str) -> None:
    outlined_poly(c, [(12, 2), (21, 9), (17, 19), (12, 22), (7, 19), (3, 9)], color, DEEP_OUTLINE, 2)
    c.polygon([(12, 4), (18, 9), (12, 18), (6, 9)], bright)
    c.polygon([(12, 4), (12, 18), (18, 9)], shadow)
    c.line([(6, 9), (18, 9)], "#FFFFFF", 2)


def badge_dragons(c: PixelCanvas, color: str, bright: str, shadow: str) -> None:
    left = [(11, 20), (6, 19), (3, 15), (6, 13), (2, 9), (7, 8), (5, 4), (10, 7), (12, 11)]
    right = [(13, 20), (18, 19), (21, 15), (18, 13), (22, 9), (17, 8), (19, 4), (14, 7), (12, 11)]
    outlined_poly(c, left, color, DEEP_OUTLINE, 2)
    outlined_poly(c, right, shadow, DEEP_OUTLINE, 2)
    c.rect((6, 9, 9, 11), bright)
    c.rect((15, 9, 18, 11), bright)
    diamond(c, 12, 14, 3, "#FFFFFF", shadow)


def badge_gate(c: PixelCanvas, color: str, bright: str, shadow: str) -> None:
    c.ellipse((3, 2, 21, 21), DEEP_OUTLINE)
    c.ellipse((5, 4, 19, 20), color)
    c.ellipse((8, 7, 16, 20), shadow)
    c.rect((3, 10, 7, 21), OUTLINE)
    c.rect((17, 10, 21, 21), OUTLINE)
    c.rect((5, 11, 7, 20), bright)
    c.rect((17, 11, 19, 20), bright)
    c.rect((7, 19, 17, 22), OUTLINE)
    c.rect((8, 19, 16, 20), bright)
    sparkle(c, 12, 10, "#FFFFFF", 2)


def badge_champion(c: PixelCanvas) -> None:
    # Core sits left/below center so the upper-right overlay quadrant remains non-critical.
    outer = [(10, 1), (13, 8), (20, 6), (16, 12), (21, 17), (14, 16),
             (11, 23), (8, 17), (2, 20), (5, 13), (1, 9), (8, 8)]
    outlined_poly(c, outer, "#3F8CFF", DEEP_OUTLINE, 2)
    inner = [(10, 5), (12, 10), (17, 10), (13, 13), (14, 17), (10, 15), (6, 17), (7, 12), (4, 10), (9, 10)]
    c.polygon(inner, "#FFD75E")
    c.rect((8, 10, 12, 14), "#FFF7B0")


def generate_badge(number: int) -> Image.Image:
    c = badge_canvas()
    silver = ("#B5C8DC", "#F1F7FF", "#697D92")
    gold = ("#D39A28", "#FBE27A", "#87571E")
    crystal = ("#65CBEA", "#D7FAFF", "#3676BD")
    if number == 1:
        badge_laurel(c, silver[0], silver[1])
    elif number == 2:
        badge_shield(c, silver[0], silver[1], silver[2])
    elif number == 3:
        badge_crown(c, silver[0], silver[1], "#D75CFF")
    elif number == 4:
        badge_wings(c, gold[0], gold[1])
    elif number == 5:
        badge_constellation(c, gold[0], gold[1])
    elif number == 6:
        badge_phoenix(c, gold[0], gold[1], "#FF762E")
    elif number == 7:
        badge_diamond(c, crystal[0], crystal[1], crystal[2])
    elif number == 8:
        badge_dragons(c, crystal[0], crystal[1], crystal[2])
    elif number == 9:
        badge_gate(c, crystal[0], crystal[1], crystal[2])
    else:
        badge_champion(c)
    return c.image.resize((96, 96), Image.Resampling.NEAREST)


def main() -> None:
    SPRITES.mkdir(parents=True, exist_ok=True)
    BADGES.mkdir(parents=True, exist_ok=True)
    for provider in PROVIDERS:
        for tier in TIERS:
            path = SPRITES / f"{provider}-rank-{tier}.png"
            generate_atlas(provider, tier).save(path, format="PNG", optimize=True)
            print(path.relative_to(ROOT))
    for number in range(1, 11):
        path = BADGES / f"prestige-{number}.png"
        generate_badge(number).save(path, format="PNG", optimize=True)
        print(path.relative_to(ROOT))


if __name__ == "__main__":
    main()
