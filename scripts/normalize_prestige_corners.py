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
    validate_top_left_corner(top_left)
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
    corners = reflected_corners(normalize_top_left_corner(source))

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
