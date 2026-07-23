#!/usr/bin/env python3
"""Normalize authored aura grids into anchored horizontal RGBA atlases."""

from __future__ import annotations

import argparse
import os
import tempfile
from pathlib import Path

from PIL import Image


AURA_CELL_SIZE = 192
AURA_BASELINE_Y = 184
TRANSPARENT_EDGE = 7
ALPHA_THRESHOLD = 16


def _rounded_boundaries(length: int, count: int) -> tuple[int, ...]:
    return tuple(round(index * length / count) for index in range(count + 1))


def _threshold_alpha(source: Image.Image) -> Image.Image:
    rgba = source.convert("RGBA")
    alpha = rgba.getchannel("A").point(
        lambda value: 0 if value <= ALPHA_THRESHOLD else value
    )
    rgba.putalpha(alpha)
    return rgba


def normalize_aura_frames(
    source: Image.Image,
    columns: int,
    rows: int,
    frame_count: int,
) -> Image.Image:
    """Return a horizontal strip of 192px cells sharing one scale and anchor."""
    if columns <= 0 or rows <= 0:
        raise ValueError("columns and rows must be positive")
    if frame_count <= 0:
        raise ValueError("frame_count must be positive")
    if frame_count > columns * rows:
        raise ValueError("frame_count exceeds grid capacity")
    if source.width < columns or source.height < rows:
        raise ValueError("source grid cells must be non-empty")

    rgba = _threshold_alpha(source)
    x_bounds = _rounded_boundaries(rgba.width, columns)
    y_bounds = _rounded_boundaries(rgba.height, rows)
    trimmed_frames: list[Image.Image] = []
    for index in range(frame_count):
        column = index % columns
        row = index // columns
        cell = rgba.crop(
            (
                x_bounds[column],
                y_bounds[row],
                x_bounds[column + 1],
                y_bounds[row + 1],
            )
        )
        bounds = cell.getchannel("A").getbbox()
        if bounds is None:
            raise ValueError(f"frame {index} has no visible pixels above alpha 16")
        trimmed_frames.append(cell.crop(bounds))

    maximum_width = max(frame.width for frame in trimmed_frames)
    maximum_height = max(frame.height for frame in trimmed_frames)
    available_width = AURA_CELL_SIZE - TRANSPARENT_EDGE * 2
    available_height = AURA_BASELINE_Y - TRANSPARENT_EDGE + 1
    shared_scale = min(
        available_width / maximum_width,
        available_height / maximum_height,
    )
    if shared_scale <= 0:
        raise ValueError("authored frames cannot fit the aura cell")

    atlas = Image.new(
        "RGBA",
        (AURA_CELL_SIZE * frame_count, AURA_CELL_SIZE),
        (0, 0, 0, 0),
    )
    for index, authored in enumerate(trimmed_frames):
        resized_size = (
            max(1, round(authored.width * shared_scale)),
            max(1, round(authored.height * shared_scale)),
        )
        resized = authored.resize(resized_size, Image.Resampling.NEAREST)
        resized_bounds = resized.getchannel("A").getbbox()
        if resized_bounds is None:
            raise ValueError(f"frame {index} disappeared during normalization")
        resized = resized.crop(resized_bounds)
        left = index * AURA_CELL_SIZE + (AURA_CELL_SIZE - resized.width) // 2
        top = AURA_BASELINE_Y - resized.height + 1
        if left <= index * AURA_CELL_SIZE or top <= 0:
            raise ValueError(f"frame {index} does not preserve transparent edges")
        atlas.alpha_composite(resized, (left, top))

    return atlas


def write_normalized_atlas(atlas: Image.Image, destination: Path) -> None:
    """Publish one validated atlas atomically without replacing it on failure."""
    if atlas.mode != "RGBA":
        raise ValueError("normalized atlas must use RGBA mode")
    if atlas.height != AURA_CELL_SIZE or atlas.width % AURA_CELL_SIZE:
        raise ValueError("normalized atlas must contain horizontal 192x192 cells")

    destination.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{destination.name}.",
        suffix=".tmp",
        dir=destination.parent,
    )
    os.close(descriptor)
    temporary = Path(temporary_name)
    try:
        atlas.save(temporary, format="PNG", optimize=True)
        with Image.open(temporary) as staged:
            staged.load()
            if staged.mode != "RGBA" or staged.size != atlas.size:
                raise OSError("staged aura atlas failed validation")
        with temporary.open("rb") as staged_file:
            os.fsync(staged_file.fileno())
        os.replace(temporary, destination)
    finally:
        temporary.unlink(missing_ok=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--columns", type=int, required=True)
    parser.add_argument("--rows", type=int, required=True)
    parser.add_argument("--frames", type=int, required=True)
    args = parser.parse_args(argv)

    if not args.input.is_file():
        parser.error(f"input file does not exist: {args.input}")
    try:
        with Image.open(args.input) as source:
            atlas = normalize_aura_frames(
                source,
                columns=args.columns,
                rows=args.rows,
                frame_count=args.frames,
            )
        write_normalized_atlas(atlas, args.output)
    except (OSError, ValueError) as error:
        parser.error(str(error))

    print(f"wrote {args.frames} aura frames to {args.output}")
    return 0


if __name__ == "__main__":
    main()
