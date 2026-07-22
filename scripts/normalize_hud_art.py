#!/usr/bin/env python3
"""Normalize generated meter and badge artwork to Mana's HUD contracts."""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


ALPHA_THRESHOLD = 16
METER_SIZE = (288, 40)
METER_INNER_SIZE = (284, 36)
BADGE_SIZE = (96, 96)
BADGE_INNER_SIZE = (88, 88)


def _normalize(
    source: Image.Image,
    *,
    output_size: tuple[int, int],
    inner_size: tuple[int, int],
) -> Image.Image:
    rgba = source.convert("RGBA")
    visible_bounds = rgba.getchannel("A").point(
        lambda alpha: 255 if alpha > ALPHA_THRESHOLD else 0
    ).getbbox()
    canvas = Image.new("RGBA", output_size, (0, 0, 0, 0))
    if visible_bounds is None:
        return canvas

    trimmed = rgba.crop(visible_bounds)
    scale = min(
        inner_size[0] / trimmed.width,
        inner_size[1] / trimmed.height,
    )
    resized_size = (
        max(1, round(trimmed.width * scale)),
        max(1, round(trimmed.height * scale)),
    )
    resized = trimmed.resize(resized_size, Image.Resampling.LANCZOS)
    offset = (
        (output_size[0] - resized.width) // 2,
        (output_size[1] - resized.height) // 2,
    )
    canvas.alpha_composite(resized, offset)
    return canvas


def normalize_meter(source: Image.Image) -> Image.Image:
    rgba = source.convert("RGBA")
    visible_bounds = rgba.getchannel("A").point(
        lambda alpha: 255 if alpha > ALPHA_THRESHOLD else 0
    ).getbbox()
    canvas = Image.new("RGBA", METER_SIZE, (0, 0, 0, 0))
    if visible_bounds is None:
        return canvas

    trimmed = rgba.crop(visible_bounds)
    resized = trimmed.resize(METER_INNER_SIZE, Image.Resampling.LANCZOS)
    canvas.alpha_composite(resized, (2, 2))
    return canvas


def normalize_badge(source: Image.Image) -> Image.Image:
    return _normalize(
        source,
        output_size=BADGE_SIZE,
        inner_size=BADGE_INNER_SIZE,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--kind", choices=("meter", "badge"), required=True)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    normalizer = normalize_meter if args.kind == "meter" else normalize_badge
    with Image.open(args.input) as source:
        result = normalizer(source)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    result.save(args.output, format="PNG", optimize=True)


if __name__ == "__main__":
    main()
