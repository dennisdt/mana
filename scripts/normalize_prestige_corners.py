#!/usr/bin/env python3
"""Normalize one authored prestige corner into a connected reflected corner set."""

from __future__ import annotations

import argparse
import os
import shutil
import tempfile
import uuid
from collections import deque
from pathlib import Path

from PIL import Image


ALPHA_THRESHOLD = 16
CANVAS_SIZE = 96
TRANSPARENT_EDGE = 4
# Generated joints occupy this bottom-right-anchored square so their rail
# centers land inside the frame's right and bottom socket windows.
NORMALIZED_ART_EXTENT = 76
RIGHT_SOCKET = (88, 28, 96, 68)
BOTTOM_SOCKET = (28, 88, 68, 96)
PRODUCTION_SCALE = 2
PRODUCTION_CORNER_SIZE = CANVAS_SIZE // PRODUCTION_SCALE
PRODUCTION_RAIL_THICKNESS = 32 // PRODUCTION_SCALE
PRODUCTION_RAIL_INSET = 16
PRODUCTION_RAIL_UNDERLAP = 8
SOCKET_ALIGNMENT_LIMIT = 32
CORNER_NAMES = ("corner-tl", "corner-tr", "corner-bl", "corner-br")
KIT_SPECS = {
    "corner-tl.png": (96, 96),
    "rail-h.png": (128, 32),
    "corner-tr.png": (96, 96),
    "rail-v.png": (32, 128),
    "crest-top.png": (192, 96),
    "corner-bl.png": (96, 96),
    "corner-br.png": (96, 96),
}


def _threshold_alpha(source: Image.Image) -> Image.Image:
    rgba = source.convert("RGBA")
    alpha = rgba.getchannel("A").point(
        lambda value: 0 if value <= ALPHA_THRESHOLD else value
    )
    rgba.putalpha(alpha)
    return rgba


def _visible_coordinates(source: Image.Image) -> set[tuple[int, int]]:
    alpha = source.getchannel("A")
    return {
        (x, y)
        for y in range(source.height)
        for x in range(source.width)
        if alpha.getpixel((x, y)) > ALPHA_THRESHOLD
    }


def _require_connected_art(source: Image.Image) -> None:
    visible = _visible_coordinates(source)
    if not visible:
        raise ValueError("corner art has no visible pixels")

    first = next(iter(visible))
    connected = {first}
    pending = deque((first,))
    while pending:
        x, y = pending.popleft()
        for y_offset in (-1, 0, 1):
            for x_offset in (-1, 0, 1):
                if x_offset == 0 and y_offset == 0:
                    continue
                neighbor = (x + x_offset, y + y_offset)
                if neighbor in visible and neighbor not in connected:
                    connected.add(neighbor)
                    pending.append(neighbor)

    if len(connected) != len(visible):
        raise ValueError("corner art must be one connected visible component")


def _socket_visible(alpha: Image.Image, bounds: tuple[int, int, int, int]) -> bool:
    return alpha.crop(bounds).getbbox() is not None


def _translate_corner(source: Image.Image, x_offset: int, y_offset: int) -> Image.Image:
    translated = Image.new("RGBA", source.size, (0, 0, 0, 0))
    translated.alpha_composite(source, (x_offset, y_offset))
    return translated


def _production_alpha(source: Image.Image, size: tuple[int, int]) -> Image.Image:
    return _threshold_alpha(
        source.resize(size, Image.Resampling.NEAREST)
    ).getchannel("A")


def _rail_cross_axis(
    rail: Image.Image,
    *,
    horizontal: bool,
) -> set[int]:
    size = (
        (64, PRODUCTION_RAIL_THICKNESS)
        if horizontal
        else (PRODUCTION_RAIL_THICKNESS, 64)
    )
    alpha = _production_alpha(rail, size)
    if horizontal:
        return {
            y
            for y in range(alpha.height)
            if all(
                alpha.getpixel((x, y)) > ALPHA_THRESHOLD
                for x in range(alpha.width)
            )
        }
    return {
        x
        for x in range(alpha.width)
        if all(
            alpha.getpixel((x, y)) > ALPHA_THRESHOLD
            for y in range(alpha.height)
        )
    }


def _overlap_metrics_for_axes(
    corner: Image.Image,
    horizontal_axis: set[int],
    vertical_axis: set[int],
) -> tuple[int, int, int]:
    """Return horizontal/vertical underlap coverage and intersecting pixels."""
    corner_alpha = _production_alpha(
        corner,
        (PRODUCTION_CORNER_SIZE, PRODUCTION_CORNER_SIZE),
    )
    horizontal_columns: set[int] = set()
    vertical_rows: set[int] = set()
    intersecting_pixels = 0
    underlap_end = PRODUCTION_CORNER_SIZE
    underlap_start = underlap_end - PRODUCTION_RAIL_UNDERLAP

    for x in range(underlap_start, underlap_end):
        for rail_y in horizontal_axis:
            y = PRODUCTION_RAIL_INSET + rail_y
            if corner_alpha.getpixel((x, y)) > ALPHA_THRESHOLD:
                horizontal_columns.add(x)
                intersecting_pixels += 1

    for y in range(underlap_start, underlap_end):
        for rail_x in vertical_axis:
            x = PRODUCTION_RAIL_INSET + rail_x
            if corner_alpha.getpixel((x, y)) > ALPHA_THRESHOLD:
                vertical_rows.add(y)
                intersecting_pixels += 1

    return (
        len(horizontal_columns),
        len(vertical_rows),
        intersecting_pixels,
    )


def _rail_overlap_metrics(
    corner: Image.Image,
    horizontal_rail: Image.Image,
    vertical_rail: Image.Image,
) -> tuple[int, int, int]:
    horizontal_axis = _rail_cross_axis(horizontal_rail, horizontal=True)
    vertical_axis = _rail_cross_axis(vertical_rail, horizontal=False)
    if not horizontal_axis:
        raise ValueError("horizontal prestige rail has no visible pixels")
    if not vertical_axis:
        raise ValueError("vertical prestige rail has no visible pixels")
    return _overlap_metrics_for_axes(corner, horizontal_axis, vertical_axis)


def _validate_corner_rail_overlap(
    corner: Image.Image,
    horizontal_rail: Image.Image,
    vertical_rail: Image.Image,
) -> None:
    horizontal, vertical, _ = _rail_overlap_metrics(
        corner,
        horizontal_rail,
        vertical_rail,
    )
    if horizontal != PRODUCTION_RAIL_UNDERLAP:
        raise ValueError(
            "top-left corner does not overlap the horizontal rail "
            "throughout the production underlap"
        )
    if vertical != PRODUCTION_RAIL_UNDERLAP:
        raise ValueError(
            "top-left corner does not overlap the vertical rail "
            "throughout the production underlap"
        )


def _align_corner_to_rails(
    corner: Image.Image,
    horizontal_rail: Image.Image,
    vertical_rail: Image.Image,
) -> Image.Image:
    """Translate a normalized joint onto its same-tier production rail masks."""
    horizontal_axis = _rail_cross_axis(horizontal_rail, horizontal=True)
    vertical_axis = _rail_cross_axis(vertical_rail, horizontal=False)
    if not horizontal_axis:
        raise ValueError("horizontal prestige rail has no visible pixels")
    if not vertical_axis:
        raise ValueError("vertical prestige rail has no visible pixels")

    offsets = range(-SOCKET_ALIGNMENT_LIMIT, SOCKET_ALIGNMENT_LIMIT + 1)
    x_offsets = [
        offset
        for offset in offsets
        if _overlap_metrics_for_axes(
            _translate_corner(corner, offset, 0),
            horizontal_axis,
            vertical_axis,
        )[1]
        == PRODUCTION_RAIL_UNDERLAP
    ]
    y_offsets = [
        offset
        for offset in offsets
        if _overlap_metrics_for_axes(
            _translate_corner(corner, 0, offset),
            horizontal_axis,
            vertical_axis,
        )[0]
        == PRODUCTION_RAIL_UNDERLAP
    ]

    candidates: list[tuple[tuple[int, int, int], int, int]] = []
    for y_offset in y_offsets:
        for x_offset in x_offsets:
            candidate = _translate_corner(corner, x_offset, y_offset)
            horizontal, vertical, overlap = _overlap_metrics_for_axes(
                candidate,
                horizontal_axis,
                vertical_axis,
            )
            if (
                horizontal != PRODUCTION_RAIL_UNDERLAP
                or vertical != PRODUCTION_RAIL_UNDERLAP
            ):
                continue
            score = (
                overlap,
                -(abs(x_offset) + abs(y_offset)),
                -max(abs(x_offset), abs(y_offset)),
            )
            candidates.append((score, x_offset, y_offset))

    for _, x_offset, y_offset in sorted(candidates, reverse=True):
        aligned = _translate_corner(corner, x_offset, y_offset)
        try:
            validate_top_left_corner(aligned)
        except ValueError:
            continue
        _validate_corner_rail_overlap(aligned, horizontal_rail, vertical_rail)
        return aligned

    raise ValueError(
        "top-left corner cannot be aligned with the same-tier frame rails"
    )


def normalize_top_left_corner(source: Image.Image) -> Image.Image:
    """Scale and anchor a connected top-left joint to Mana's 96px corner contract."""
    rgba = _threshold_alpha(source)
    _require_connected_art(rgba)
    visible_bounds = rgba.getchannel("A").getbbox()
    if visible_bounds is None:
        raise ValueError("corner art has no visible pixels")

    trimmed = rgba.crop(visible_bounds)
    available = NORMALIZED_ART_EXTENT
    scale = min(available / trimmed.width, available / trimmed.height)
    resized_size = (
        min(available, max(1, round(trimmed.width * scale))),
        min(available, max(1, round(trimmed.height * scale))),
    )
    resized = trimmed.resize(resized_size, Image.Resampling.NEAREST)

    canvas = Image.new(
        "RGBA",
        (CANVAS_SIZE, CANVAS_SIZE),
        (0, 0, 0, 0),
    )
    canvas.alpha_composite(
        resized,
        (CANVAS_SIZE - resized.width, CANVAS_SIZE - resized.height),
    )
    validate_top_left_corner(canvas)
    return canvas


def validate_top_left_corner(corner: Image.Image) -> None:
    """Reject corners that cannot join the top and left frame rails cleanly."""
    if corner.size != (CANVAS_SIZE, CANVAS_SIZE):
        raise ValueError("top-left corner must be exactly 96x96 pixels")
    if corner.mode != "RGBA":
        raise ValueError("top-left corner must use RGBA pixels")

    normalized = _threshold_alpha(corner)
    alpha = normalized.getchannel("A")
    if alpha.crop((0, 0, TRANSPARENT_EDGE, CANVAS_SIZE)).getbbox() is not None:
        raise ValueError("top-left corner exterior left band must be transparent")
    if alpha.crop((0, 0, CANVAS_SIZE, TRANSPARENT_EDGE)).getbbox() is not None:
        raise ValueError("top-left corner exterior top band must be transparent")
    if not _socket_visible(alpha, RIGHT_SOCKET):
        raise ValueError("top-left corner does not reach the right rail socket")
    if not _socket_visible(alpha, BOTTOM_SOCKET):
        raise ValueError("top-left corner does not reach the bottom rail socket")
    _require_connected_art(normalized)


def reflected_corners(top_left: Image.Image) -> dict[str, Image.Image]:
    """Derive every other corner as an exact pixel reflection of top-left."""
    validate_top_left_corner(top_left)
    top_left = top_left.copy()
    return {
        "corner-tl": top_left,
        "corner-tr": top_left.transpose(Image.Transpose.FLIP_LEFT_RIGHT),
        "corner-bl": top_left.transpose(Image.Transpose.FLIP_TOP_BOTTOM),
        "corner-br": top_left.transpose(
            Image.Transpose.FLIP_LEFT_RIGHT
        ).transpose(Image.Transpose.FLIP_TOP_BOTTOM),
    }


def _clean_directory(directory: Path) -> None:
    shutil.rmtree(directory, ignore_errors=True)


def _publish_staged_directory(staging: Path, destination: Path) -> None:
    """Replace a complete destination with staged output, restoring it on failure."""
    backup = destination.with_name(f".{destination.name}.backup-{uuid.uuid4().hex}")
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


def _require_regular_kit(destination: Path) -> None:
    if not destination.is_dir() or destination.is_symlink():
        raise ValueError(f"prestige kit does not exist: {destination}")
    symlinks = [path for path in destination.rglob("*") if path.is_symlink()]
    if symlinks:
        raise ValueError(f"prestige kit contains a symbolic link: {symlinks[0]}")
    for filename in KIT_SPECS:
        path = destination / filename
        if not path.is_file():
            raise ValueError(f"prestige kit is missing {filename}")


def _file_bytes(directory: Path, *, exclude_corners: bool) -> dict[Path, bytes]:
    corner_files = {f"{name}.png" for name in CORNER_NAMES}
    files: dict[Path, bytes] = {}
    for path in directory.rglob("*"):
        if not path.is_file():
            continue
        relative_path = path.relative_to(directory)
        is_root_corner = len(relative_path.parts) == 1 and path.name in corner_files
        if not exclude_corners or not is_root_corner:
            files[relative_path] = path.read_bytes()
    return files


def _validate_staged_kit(
    staging: Path,
    expected_non_corner_bytes: dict[Path, bytes],
) -> None:
    for path in staging.rglob("*.png"):
        if path.is_symlink():
            raise ValueError(f"staged prestige asset is a symbolic link: {path.name}")
        try:
            with Image.open(path) as image:
                if image.format != "PNG":
                    raise ValueError(
                        f"staged prestige asset is not PNG: {path.name}"
                    )
                image.load()
        except OSError as error:
            raise ValueError(
                f"staged prestige asset is unreadable: {path.name}"
            ) from error

    for filename, expected_size in KIT_SPECS.items():
        path = staging / filename
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"staged prestige kit is missing {filename}")
        with Image.open(path) as image:
            if image.size != expected_size:
                raise ValueError(f"staged prestige asset has wrong size: {filename}")

    staged_non_corner_bytes = _file_bytes(staging, exclude_corners=True)
    if staged_non_corner_bytes != expected_non_corner_bytes:
        raise ValueError("staging changed a non-corner prestige asset")

    with Image.open(staging / "corner-tl.png") as image:
        top_left = image.convert("RGBA")
    with Image.open(staging / "rail-h.png") as image:
        horizontal_rail = image.convert("RGBA")
    with Image.open(staging / "rail-v.png") as image:
        vertical_rail = image.convert("RGBA")
    validate_top_left_corner(top_left)
    _validate_corner_rail_overlap(top_left, horizontal_rail, vertical_rail)
    expected = reflected_corners(top_left)
    for name, reflected in expected.items():
        with Image.open(staging / f"{name}.png") as image:
            actual = image.convert("RGBA")
        if actual.tobytes() != reflected.tobytes():
            raise ValueError(f"{name}.png is not an exact top-left reflection")


def publish_prestige_corners(
    source: Image.Image,
    prestige: int,
    output_root: Path,
) -> Path:
    """Replace only one prestige kit's corners through a complete atomic swap."""
    if (
        isinstance(prestige, bool)
        or not isinstance(prestige, int)
        or not 1 <= prestige <= 10
    ):
        raise ValueError("prestige must be an integer from 1 through 10")

    output_root = Path(output_root)
    destination = output_root / "prestige" / str(prestige)
    _require_regular_kit(destination)
    expected_non_corner_bytes = _file_bytes(destination, exclude_corners=True)
    with Image.open(destination / "rail-h.png") as image:
        horizontal_rail = image.convert("RGBA")
    with Image.open(destination / "rail-v.png") as image:
        vertical_rail = image.convert("RGBA")
    top_left = _align_corner_to_rails(
        normalize_top_left_corner(source),
        horizontal_rail,
        vertical_rail,
    )
    corners = reflected_corners(top_left)

    staging = Path(
        tempfile.mkdtemp(
            prefix=f".{destination.name}.staging-",
            dir=destination.parent,
        )
    )
    try:
        shutil.copytree(
            destination,
            staging,
            dirs_exist_ok=True,
            copy_function=shutil.copy2,
        )
        for name, corner in corners.items():
            corner.save(staging / f"{name}.png", format="PNG", optimize=True)
        _validate_staged_kit(staging, expected_non_corner_bytes)
    except (OSError, ValueError):
        _clean_directory(staging)
        raise

    _publish_staged_directory(staging, destination)
    return destination


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Normalize and publish one connected prestige corner set."
    )
    parser.add_argument("--prestige", type=int, required=True)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    args = parser.parse_args(argv)

    if not args.input.is_file():
        parser.error(f"input file does not exist: {args.input}")
    try:
        with Image.open(args.input) as source:
            destination = publish_prestige_corners(
                source,
                args.prestige,
                args.output_root,
            )
    except (OSError, ValueError) as error:
        parser.error(str(error))

    print(f"wrote prestige {args.prestige} corners to {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
