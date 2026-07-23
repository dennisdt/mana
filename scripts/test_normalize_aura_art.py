#!/usr/bin/env python3
"""Focused tests for generated elemental-aura normalization."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))

from normalize_aura_art import (
    AURA_BASELINE_Y,
    AURA_CELL_SIZE,
    normalize_aura_frames,
    write_normalized_atlas,
)


def proportional_bounds(length: int, count: int) -> list[int]:
    return [round(index * length / count) for index in range(count + 1)]


def make_grid(
    sizes: list[tuple[int, int]],
    *,
    columns: int,
    rows: int,
    source_size: tuple[int, int] = (307, 211),
) -> Image.Image:
    image = Image.new("RGBA", source_size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    x_bounds = proportional_bounds(source_size[0], columns)
    y_bounds = proportional_bounds(source_size[1], rows)
    for index, (subject_width, subject_height) in enumerate(sizes):
        column = index % columns
        row = index // columns
        left = x_bounds[column] + 7
        top = y_bounds[row] + 5
        color = (30 + index * 31, 80 + index * 23, 170 - index * 17, 255)
        draw.rectangle(
            (
                left,
                top,
                left + subject_width - 1,
                top + subject_height - 1,
            ),
            fill=color,
        )
        draw.point((x_bounds[column] + 1, y_bounds[row] + 1), fill=(255, 0, 255, 16))
    return image


def frame(atlas: Image.Image, index: int) -> Image.Image:
    return atlas.crop(
        (
            index * AURA_CELL_SIZE,
            0,
            (index + 1) * AURA_CELL_SIZE,
            AURA_CELL_SIZE,
        )
    )


def edge_alpha(image: Image.Image) -> list[int]:
    alpha = image.getchannel("A")
    width, height = image.size
    return (
        [alpha.getpixel((x, 0)) for x in range(width)]
        + [alpha.getpixel((x, height - 1)) for x in range(width)]
        + [alpha.getpixel((0, y)) for y in range(height)]
        + [alpha.getpixel((width - 1, y)) for y in range(height)]
    )


def visible_bounds(image: Image.Image) -> tuple[int, int, int, int]:
    bounds = image.getchannel("A").getbbox()
    if bounds is None:
        raise AssertionError("expected a visible frame")
    return bounds


class AuraArtNormalizationTests(unittest.TestCase):
    def test_normalizes_non_divisible_grid_with_shared_scale_center_and_baseline(
        self,
    ) -> None:
        sizes = [(18, 22), (29, 31), (43, 37), (35, 50)]
        result = normalize_aura_frames(
            make_grid(sizes, columns=3, rows=2),
            columns=3,
            rows=2,
            frame_count=4,
        )

        self.assertEqual(result.mode, "RGBA")
        self.assertEqual(result.size, (AURA_CELL_SIZE * 4, AURA_CELL_SIZE))

        normalized_sizes: list[tuple[int, int]] = []
        for index in range(4):
            normalized = frame(result, index)
            left, top, right, bottom = visible_bounds(normalized)
            normalized_sizes.append((right - left, bottom - top))
            self.assertEqual(max(edge_alpha(normalized)), 0)
            self.assertEqual(bottom - 1, AURA_BASELINE_Y)
            self.assertLessEqual(abs((left + right) / 2 - AURA_CELL_SIZE / 2), 0.5)

        scale_estimates = [
            normalized_width / source_width
            for (normalized_width, _), (source_width, _) in zip(normalized_sizes, sizes)
        ] + [
            normalized_height / source_height
            for (_, normalized_height), (_, source_height) in zip(normalized_sizes, sizes)
        ]
        self.assertLessEqual(max(scale_estimates) - min(scale_estimates), 0.12)

    def test_alpha_threshold_uses_strictly_greater_than_sixteen(self) -> None:
        source = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
        source.putpixel((2, 2), (255, 0, 255, 16))
        source.putpixel((20, 20), (20, 40, 60, 17))

        result = normalize_aura_frames(source, columns=1, rows=1, frame_count=1)

        visible = frame(result, 0)
        self.assertIsNotNone(visible.getchannel("A").getbbox())
        self.assertEqual(
            {
                alpha
                for alpha in visible.getchannel("A").get_flattened_data()
                if alpha
            },
            {17},
        )

    def test_grid_split_uses_rounded_proportional_cell_boundaries(self) -> None:
        source = Image.new("RGBA", (10, 9), (0, 0, 0, 0))
        source.putpixel((1, 4), (220, 30, 30, 255))
        source.putpixel((6, 4), (30, 220, 30, 255))
        source.putpixel((8, 4), (30, 30, 220, 255))

        result = normalize_aura_frames(source, columns=3, rows=1, frame_count=3)

        center = AURA_CELL_SIZE // 2
        self.assertEqual(
            frame(result, 0).getpixel((center, AURA_BASELINE_Y))[:3],
            (220, 30, 30),
        )
        self.assertEqual(
            frame(result, 1).getpixel((center, AURA_BASELINE_Y))[:3],
            (30, 220, 30),
        )
        self.assertEqual(
            frame(result, 2).getpixel((center, AURA_BASELINE_Y))[:3],
            (30, 30, 220),
        )

    def test_rejects_invalid_geometry_and_empty_authored_frames(self) -> None:
        source = Image.new("RGBA", (20, 20), (0, 0, 0, 0))
        with self.assertRaisesRegex(ValueError, "columns and rows must be positive"):
            normalize_aura_frames(source, columns=0, rows=1, frame_count=1)
        with self.assertRaisesRegex(ValueError, "frame_count must be positive"):
            normalize_aura_frames(source, columns=1, rows=1, frame_count=0)
        with self.assertRaisesRegex(ValueError, "exceeds grid capacity"):
            normalize_aura_frames(source, columns=1, rows=1, frame_count=2)
        with self.assertRaisesRegex(ValueError, "frame 0 has no visible pixels"):
            normalize_aura_frames(source, columns=1, rows=1, frame_count=1)

    def test_write_publishes_one_complete_atlas_atomically(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            destination = Path(temp_directory) / "effects" / "aura.png"
            destination.parent.mkdir()
            destination.write_bytes(b"previous atlas")
            normalized = normalize_aura_frames(
                make_grid([(18, 22), (29, 31)], columns=2, rows=1),
                columns=2,
                rows=1,
                frame_count=2,
            )

            write_normalized_atlas(normalized, destination)

            with Image.open(destination) as written:
                self.assertEqual(written.size, (AURA_CELL_SIZE * 2, AURA_CELL_SIZE))
                self.assertEqual(written.mode, "RGBA")
            self.assertEqual(list(destination.parent.glob(".aura.png.*")), [])

    def test_write_failure_preserves_previous_atlas_and_removes_staging_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            destination = Path(temp_directory) / "effects" / "aura.png"
            destination.parent.mkdir()
            previous = b"previous complete atlas"
            destination.write_bytes(previous)
            normalized = Image.new(
                "RGBA",
                (AURA_CELL_SIZE, AURA_CELL_SIZE),
                (0, 0, 0, 0),
            )

            with mock.patch.object(
                Image.Image,
                "save",
                side_effect=OSError("simulated save failure"),
            ):
                with self.assertRaisesRegex(OSError, "simulated save failure"):
                    write_normalized_atlas(normalized, destination)

            self.assertEqual(destination.read_bytes(), previous)
            self.assertEqual(list(destination.parent.glob(".aura.png.*")), [])

    def test_replace_failure_preserves_previous_atlas_and_removes_staging_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            destination = Path(temp_directory) / "effects" / "aura.png"
            destination.parent.mkdir()
            previous = b"previous complete atlas"
            destination.write_bytes(previous)
            normalized = Image.new(
                "RGBA",
                (AURA_CELL_SIZE, AURA_CELL_SIZE),
                (0, 0, 0, 0),
            )

            with mock.patch(
                "normalize_aura_art.os.replace",
                side_effect=OSError("simulated replace failure"),
            ):
                with self.assertRaisesRegex(OSError, "simulated replace failure"):
                    write_normalized_atlas(normalized, destination)

            self.assertEqual(destination.read_bytes(), previous)
            self.assertEqual(list(destination.parent.glob(".aura.png.*")), [])

    def test_cli_writes_requested_horizontal_strip(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "source.png"
            destination = root / "effects" / "aura.png"
            make_grid([(18, 22), (29, 31)], columns=2, rows=1).save(source)

            completed = subprocess.run(
                [
                    sys.executable,
                    str(Path(__file__).with_name("normalize_aura_art.py")),
                    "--input",
                    str(source),
                    "--output",
                    str(destination),
                    "--columns",
                    "2",
                    "--rows",
                    "1",
                    "--frames",
                    "2",
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            with Image.open(destination) as written:
                self.assertEqual(written.size, (AURA_CELL_SIZE * 2, AURA_CELL_SIZE))
                self.assertEqual(written.mode, "RGBA")


if __name__ == "__main__":
    unittest.main()
