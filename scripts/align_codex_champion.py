#!/usr/bin/env python3
"""Optically align Codex Champion idle/hover rows to Claude Champion."""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


BASE_ATLAS_SIZE = (448, 336)
BASE_CELL_SIZE = 112
BASE_CELL_MARGIN = 4
ALPHA_THRESHOLD = 16
ROW_TARGETS = {
    # The targets are the accepted post-scale heights for each animation
    # frame. They correspond to the original ~82/64 idle and 84/60 hover
    # optical correction while keeping every frame within 4px of Claude.
    0: ([83, 81, 84, 82], 97),
    2: ([84, 84, 84, 84], 89),
}


def visible_bbox(image: Image.Image) -> tuple[int, int, int, int]:
    bounds = image.getchannel("A").point(
        lambda alpha: 255 if alpha > ALPHA_THRESHOLD else 0
    ).getbbox()
    if bounds is None:
        raise ValueError("Cannot align an empty Champion cell")
    return bounds


def rescale_cell(
    cell: Image.Image,
    *,
    target_height: int,
    target_bottom: int,
    cell_size: int = BASE_CELL_SIZE,
    cell_margin: int = BASE_CELL_MARGIN,
) -> Image.Image:
    source_bounds = visible_bbox(cell)
    source_height = source_bounds[3] - source_bounds[1]
    if source_height == target_height and source_bounds[3] == target_bottom:
        return cell.copy()

    trimmed = cell.crop(source_bounds)
    scale = target_height / trimmed.height
    resized = trimmed.resize(
        (
            max(1, round(trimmed.width * scale)),
            target_height,
        ),
        Image.Resampling.LANCZOS,
    )
    left, top, right, bottom = visible_bbox(resized)
    if bottom - top != target_height:
        raise ValueError(
            f"Resampling produced visible height {bottom - top}, "
            f"expected {target_height}"
        )
    visible_width = right - left
    offset = (
        (cell_size - visible_width) // 2 - left,
        target_bottom - bottom,
    )

    if (
        offset[0] < cell_margin
        or offset[1] < cell_margin
        or offset[0] + resized.width > cell_size - cell_margin
        or offset[1] + resized.height > cell_size - cell_margin
    ):
        raise ValueError(
            f"Scaled Champion cell violates the {cell_margin}px safety margin: "
            f"size={resized.size}, offset={offset}"
        )

    result = Image.new("RGBA", (cell_size, cell_size), (0, 0, 0, 0))
    result.alpha_composite(resized, offset)
    return result


def align_champion(atlas: Image.Image) -> Image.Image:
    rgba = atlas.convert("RGBA")
    if rgba.width % BASE_ATLAS_SIZE[0] != 0:
        raise ValueError(f"Expected a Retina multiple of 448x336, got {rgba.size}")
    retina_scale = rgba.width // BASE_ATLAS_SIZE[0]
    if retina_scale not in (1, 2) or rgba.height != BASE_ATLAS_SIZE[1] * retina_scale:
        raise ValueError(f"Expected a 2x or 4x Champion atlas, got {rgba.size}")
    cell_size = BASE_CELL_SIZE * retina_scale
    cell_margin = BASE_CELL_MARGIN * retina_scale

    working_before = rgba.crop((0, cell_size, rgba.width, cell_size * 2)).tobytes()
    result = rgba.copy()
    for row, (target_heights, target_bottom) in ROW_TARGETS.items():
        for column in range(4):
            box = (
                column * cell_size,
                row * cell_size,
                (column + 1) * cell_size,
                (row + 1) * cell_size,
            )
            corrected = rescale_cell(
                rgba.crop(box),
                target_height=target_heights[column] * retina_scale,
                target_bottom=target_bottom * retina_scale,
                cell_size=cell_size,
                cell_margin=cell_margin,
            )
            result.paste(corrected, box[:2])

    working_after = result.crop((0, cell_size, rgba.width, cell_size * 2)).tobytes()
    if working_after != working_before:
        raise AssertionError("Working row changed during Champion optical alignment")
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    with Image.open(args.input) as source:
        result = align_champion(source)
    result.save(args.output, format="PNG", optimize=True)


if __name__ == "__main__":
    main()
