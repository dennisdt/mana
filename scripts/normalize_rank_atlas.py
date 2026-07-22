#!/usr/bin/env python3
"""Normalize a generated 4x3 rank atlas to Mana's exact cell contract."""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


CELL_SIZE = 112
CELL_MARGIN = 4
ATLAS_COLUMNS = 4
ATLAS_ROWS = 3
INNER_SIZE = CELL_SIZE - CELL_MARGIN * 2


def normalize_atlas(source: Image.Image) -> Image.Image:
    rgba = source.convert("RGBA")
    atlas = Image.new(
        "RGBA",
        (CELL_SIZE * ATLAS_COLUMNS, CELL_SIZE * ATLAS_ROWS),
        (0, 0, 0, 0),
    )

    for row in range(ATLAS_ROWS):
        row_cells: list[Image.Image] = []
        for column in range(ATLAS_COLUMNS):
            left = round(column * rgba.width / ATLAS_COLUMNS)
            right = round((column + 1) * rgba.width / ATLAS_COLUMNS)
            top = round(row * rgba.height / ATLAS_ROWS)
            bottom = round((row + 1) * rgba.height / ATLAS_ROWS)
            cell = rgba.crop((left, top, right, bottom)).resize(
                (INNER_SIZE, INNER_SIZE), Image.Resampling.LANCZOS
            )
            padded = Image.new("RGBA", (CELL_SIZE, CELL_SIZE), (0, 0, 0, 0))
            padded.alpha_composite(cell, (CELL_MARGIN, CELL_MARGIN))
            row_cells.append(padded)

        bottoms = []
        for cell in row_cells:
            bbox = cell.getchannel("A").point(
                lambda alpha: 255 if alpha > 16 else 0
            ).getbbox()
            bottoms.append(bbox[3] - 1 if bbox is not None else CELL_MARGIN)
        target_bottom = min(CELL_SIZE - CELL_MARGIN - 1, max(bottoms))

        for column, (cell, bottom) in enumerate(zip(row_cells, bottoms, strict=True)):
            aligned = Image.new("RGBA", (CELL_SIZE, CELL_SIZE), (0, 0, 0, 0))
            aligned.alpha_composite(cell, (0, target_bottom - bottom))
            clipped = Image.new("RGBA", (CELL_SIZE, CELL_SIZE), (0, 0, 0, 0))
            clipped.alpha_composite(
                aligned.crop(
                    (
                        CELL_MARGIN,
                        CELL_MARGIN,
                        CELL_SIZE - CELL_MARGIN,
                        CELL_SIZE - CELL_MARGIN,
                    )
                ),
                (CELL_MARGIN, CELL_MARGIN),
            )
            atlas.alpha_composite(
                clipped, (column * CELL_SIZE, row * CELL_SIZE)
            )

    return atlas


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    with Image.open(args.input) as source:
        atlas = normalize_atlas(source)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(args.output, format="PNG", optimize=True)


if __name__ == "__main__":
    main()
