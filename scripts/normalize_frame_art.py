#!/usr/bin/env python3
"""Normalize generated 3x3 frame kits to Mana's bitmap contracts."""

from __future__ import annotations

import argparse
import os
import shutil
import tempfile
import uuid
from pathlib import Path

from PIL import Image, ImageOps


ALPHA_THRESHOLD = 16
TRANSPARENT_EDGE = 4
PIECE_SPECS = {
    "corner-tl": (96, 96),
    "rail-h": (128, 32),
    "corner-tr": (96, 96),
    "rail-v": (32, 128),
    "crest-top": (192, 96),
    "ornament-h": (64, 32),
    "corner-bl": (96, 96),
    "ornament-v": (32, 64),
    "corner-br": (96, 96),
}
REQUIRED_PIECES = {
    "rank": (
        "corner-tl",
        "rail-h",
        "corner-tr",
        "rail-v",
        "corner-bl",
        "corner-br",
    ),
    "prestige": (
        "corner-tl",
        "rail-h",
        "corner-tr",
        "rail-v",
        "crest-top",
        "corner-bl",
        "corner-br",
    ),
}
RANK_NAMES = frozenset(
    {
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
    }
)


def _threshold_alpha(source: Image.Image) -> Image.Image:
    rgba = source.convert("RGBA")
    alpha = rgba.getchannel("A").point(
        lambda value: 0 if value < ALPHA_THRESHOLD else value
    )
    rgba.putalpha(alpha)
    return rgba


def split_frame_kit(source: Image.Image) -> dict[str, Image.Image]:
    """Split a source image into its fixed row-major three-by-three cells."""
    width, height = source.size
    if width % 3 or height % 3:
        raise ValueError("frame-kit width and height must be divisible by 3")

    cell_width, cell_height = width // 3, height // 3
    if cell_width == 0 or cell_height == 0:
        raise ValueError("frame-kit cells must be non-empty")

    rgba = source.convert("RGBA")
    return {
        name: rgba.crop(
            (
                (index % 3) * cell_width,
                (index // 3) * cell_height,
                (index % 3 + 1) * cell_width,
                (index // 3 + 1) * cell_height,
            )
        )
        for index, name in enumerate(PIECE_SPECS)
    }


def normalize_piece(source: Image.Image, size: tuple[int, int]) -> Image.Image:
    """Trim and center one frame piece inside its transparent target canvas."""
    if size[0] <= TRANSPARENT_EDGE * 2 or size[1] <= TRANSPARENT_EDGE * 2:
        raise ValueError("target size must leave a transparent edge")

    rgba = _threshold_alpha(source)
    visible_bounds = rgba.getchannel("A").getbbox()
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    if visible_bounds is None:
        return canvas

    trimmed = rgba.crop(visible_bounds)
    inner_size = (size[0] - TRANSPARENT_EDGE * 2, size[1] - TRANSPARENT_EDGE * 2)
    scale = min(inner_size[0] / trimmed.width, inner_size[1] / trimmed.height)
    resized_size = (
        max(1, round(trimmed.width * scale)),
        max(1, round(trimmed.height * scale)),
    )
    resized = trimmed.resize(resized_size, Image.Resampling.NEAREST)
    offset = ((size[0] - resized.width) // 2, (size[1] - resized.height) // 2)
    canvas.alpha_composite(resized, offset)
    return canvas


def _central_visible_run(source: Image.Image, *, horizontal: bool) -> tuple[int, int]:
    alpha = source.getchannel("A")
    primary_size = source.width if horizontal else source.height
    cross_size = source.height if horizontal else source.width
    visible = []
    for position in range(primary_size):
        bounds = (
            (position, 0, position + 1, cross_size)
            if horizontal
            else (0, position, cross_size, position + 1)
        )
        visible.append(alpha.crop(bounds).getbbox() is not None)

    runs: list[tuple[int, int]] = []
    start: int | None = None
    for position, is_visible in enumerate([*visible, False]):
        if is_visible and start is None:
            start = position
        elif not is_visible and start is not None:
            runs.append((start, position))
            start = None

    if not runs:
        raise ValueError("rail has no visible authored segment")

    center = primary_size / 2
    return min(
        runs,
        key=lambda run: (
            0 if run[0] <= center < run[1] else 1,
            abs((run[0] + run[1]) / 2 - center),
            -(run[1] - run[0]),
        ),
    )


def make_repeatable_rail(source: Image.Image, *, horizontal: bool) -> Image.Image:
    """Build a seamless rail from the central authored pixels without interpolation."""
    rgba = source.convert("RGBA")
    primary_size = rgba.width if horizontal else rgba.height
    cross_size = rgba.height if horizontal else rgba.width
    if primary_size % 2:
        raise ValueError("rail primary axis must have an even size")

    run_start, run_end = _central_visible_run(rgba, horizontal=horizontal)
    half_size = primary_size // 2
    segment_size = min(half_size, run_end - run_start)
    segment_start = max(
        run_start,
        min((primary_size - segment_size) // 2, run_end - segment_size),
    )
    segment = rgba.crop(
        (segment_start, 0, segment_start + segment_size, cross_size)
        if horizontal
        else (0, segment_start, cross_size, segment_start + segment_size)
    )

    half = Image.new(
        "RGBA",
        (half_size, cross_size) if horizontal else (cross_size, half_size),
        (0, 0, 0, 0),
    )
    for offset in range(0, half_size, segment_size):
        half.paste(segment, (offset, 0) if horizontal else (0, offset))

    mirrored = ImageOps.mirror(half) if horizontal else ImageOps.flip(half)
    result = Image.new("RGBA", rgba.size, (0, 0, 0, 0))
    result.paste(half, (0, 0))
    result.paste(mirrored, (half_size, 0) if horizontal else (0, half_size))
    return result


def _target_directory(family: str, name: str, output_root: Path) -> Path:
    if family == "rank":
        if name not in RANK_NAMES:
            raise ValueError("rank name must be a supported illustrated rank")
        return output_root / "ranks" / name

    if not name.isdecimal() or not 1 <= int(name) <= 10:
        raise ValueError("prestige name must be an integer from 1 through 10")
    return output_root / "prestige" / str(int(name))


def _clean_directory(directory: Path) -> None:
    shutil.rmtree(directory, ignore_errors=True)


def _publish_staged_directory(staging: Path, destination: Path) -> None:
    """Replace a complete destination with staged output, restoring it on failure."""
    backup = destination.with_name(f".{destination.name}.backup-{uuid.uuid4().hex}")
    if not destination.exists():
        try:
            os.replace(staging, destination)
        except OSError:
            _clean_directory(staging)
            raise
        return

    try:
        os.replace(destination, backup)
    except OSError:
        _clean_directory(staging)
        raise

    try:
        os.replace(staging, destination)
    except OSError:
        try:
            os.replace(backup, destination)
        except OSError:
            # Retain the backup for manual recovery when rollback cannot complete.
            raise
        finally:
            _clean_directory(staging)
        raise

    _clean_directory(backup)


def normalize_frame_kit(
    source: Image.Image,
    *,
    family: str,
    name: str,
    output_root: Path,
) -> tuple[Path, tuple[str, ...]]:
    """Normalize a kit, validate required pieces, then write its non-empty PNGs."""
    destination = _target_directory(family, name, output_root)
    pieces = split_frame_kit(source)
    normalized = {
        piece_name: normalize_piece(piece, PIECE_SPECS[piece_name])
        for piece_name, piece in pieces.items()
    }
    empty = {
        piece_name
        for piece_name, piece in normalized.items()
        if piece.getchannel("A").getbbox() is None
    }
    missing = tuple(piece_name for piece_name in REQUIRED_PIECES[family] if piece_name in empty)
    if missing:
        raise ValueError(f"required pieces are empty: {', '.join(missing)}")

    normalized["rail-h"] = make_repeatable_rail(
        normalized["rail-h"],
        horizontal=True,
    )
    normalized["rail-v"] = make_repeatable_rail(
        normalized["rail-v"],
        horizontal=False,
    )

    destination.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.staging-", dir=destination.parent)
    )
    try:
        for piece_name in PIECE_SPECS:
            if piece_name not in empty:
                normalized[piece_name].save(
                    staging / f"{piece_name}.png", format="PNG", optimize=True
                )
    except OSError:
        _clean_directory(staging)
        raise

    _publish_staged_directory(staging, destination)
    return destination, tuple(piece_name for piece_name in PIECE_SPECS if piece_name not in empty)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--family", choices=("rank", "prestige"), required=True)
    parser.add_argument("--name", required=True)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    args = parser.parse_args(argv)

    if not args.input.is_file():
        parser.error(f"input file does not exist: {args.input}")
    try:
        with Image.open(args.input) as source:
            destination, names = normalize_frame_kit(
                source,
                family=args.family,
                name=args.name,
                output_root=args.output_root,
            )
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(f"wrote {len(names)} pieces to {destination}")
    return 0


if __name__ == "__main__":
    main()
