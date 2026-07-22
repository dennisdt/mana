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
def normalize_atlas(
    source: Image.Image,
    *,
    cell_size: int = CELL_SIZE,
    cell_margin: int = CELL_MARGIN,
    source_row_bounds: list[tuple[int, int]] | None = None,
) -> Image.Image:
    if cell_size <= 0 or cell_margin < 0 or cell_margin * 2 >= cell_size:
        raise ValueError("cell size and margin must leave a positive inner area")
    inner_size = cell_size - cell_margin * 2
    rgba = source.convert("RGBA")
    if source_row_bounds is not None:
        if len(source_row_bounds) != ATLAS_ROWS:
            raise ValueError(f"expected {ATLAS_ROWS} source row bounds")
        if any(top < 0 or bottom > rgba.height or top >= bottom for top, bottom in source_row_bounds):
            raise ValueError("source row bounds must be ordered inside the source image")
    atlas = Image.new(
        "RGBA",
        (cell_size * ATLAS_COLUMNS, cell_size * ATLAS_ROWS),
        (0, 0, 0, 0),
    )

    for row in range(ATLAS_ROWS):
        row_cells: list[Image.Image] = []
        for column in range(ATLAS_COLUMNS):
            left = round(column * rgba.width / ATLAS_COLUMNS)
            right = round((column + 1) * rgba.width / ATLAS_COLUMNS)
            if source_row_bounds is None:
                top = round(row * rgba.height / ATLAS_ROWS)
                bottom = round((row + 1) * rgba.height / ATLAS_ROWS)
            else:
                top, bottom = source_row_bounds[row]
            cell = rgba.crop((left, top, right, bottom)).resize(
                (inner_size, inner_size), Image.Resampling.LANCZOS
            )
            padded = Image.new("RGBA", (cell_size, cell_size), (0, 0, 0, 0))
            padded.alpha_composite(cell, (cell_margin, cell_margin))
            row_cells.append(padded)

        bottoms = []
        for cell in row_cells:
            bbox = cell.getchannel("A").point(
                lambda alpha: 255 if alpha > 16 else 0
            ).getbbox()
            bottoms.append(bbox[3] - 1 if bbox is not None else cell_margin)
        target_bottom = min(cell_size - cell_margin - 1, max(bottoms))

        for column, (cell, bottom) in enumerate(zip(row_cells, bottoms, strict=True)):
            aligned = Image.new("RGBA", (cell_size, cell_size), (0, 0, 0, 0))
            aligned.alpha_composite(cell, (0, target_bottom - bottom))
            clipped = Image.new("RGBA", (cell_size, cell_size), (0, 0, 0, 0))
            clipped.alpha_composite(
                aligned.crop(
                    (
                        cell_margin,
                        cell_margin,
                        cell_size - cell_margin,
                        cell_size - cell_margin,
                    )
                ),
                (cell_margin, cell_margin),
            )
            atlas.alpha_composite(
                clipped, (column * cell_size, row * cell_size)
            )

    return atlas


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cell-size", type=int, default=CELL_SIZE)
    parser.add_argument("--cell-margin", type=int, default=CELL_MARGIN)
    parser.add_argument(
        "--source-row-bounds",
        help="comma-separated top:bottom source crops for the three rows",
    )
    args = parser.parse_args()

    source_row_bounds = None
    if args.source_row_bounds:
        try:
            source_row_bounds = [
                tuple(int(value) for value in pair.split(":", 1))
                for pair in args.source_row_bounds.split(",")
            ]
        except ValueError as error:
            parser.error(f"invalid --source-row-bounds: {error}")

    with Image.open(args.input) as source:
        atlas = normalize_atlas(
            source,
            cell_size=args.cell_size,
            cell_margin=args.cell_margin,
            source_row_bounds=source_row_bounds,
        )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(args.output, format="PNG", optimize=True)


if __name__ == "__main__":
    main()
