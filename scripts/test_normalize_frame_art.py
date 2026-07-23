#!/usr/bin/env python3
"""Focused tests for generated frame-kit normalization."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))

from normalize_frame_art import PIECE_SPECS, normalize_piece, split_frame_kit


PIECE_NAMES = tuple(PIECE_SPECS)
BASE_PIECES = (
    "corner-tl",
    "rail-h",
    "corner-tr",
    "rail-v",
    "corner-bl",
    "corner-br",
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


def make_fixture_kit(*, empty: set[str] | None = None) -> Image.Image:
    empty = empty or set()
    cell_size = 36
    image = Image.new("RGBA", (cell_size * 3, cell_size * 3), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    for index, name in enumerate(PIECE_NAMES):
        if name in empty:
            continue
        column, row = index % 3, index // 3
        color = (index * 23 + 20, index * 17 + 30, index * 13 + 40, 255)
        left = column * cell_size
        top = row * cell_size
        draw.rectangle((left + 8, top + 10, left + 27, top + 25), fill=color)
        draw.point((left + 7, top + 9), fill=(255, 255, 255, 15))
    return image


class FrameArtNormalizationTests(unittest.TestCase):
    def test_split_frame_kit_uses_fixed_nine_cell_contract(self) -> None:
        pieces = split_frame_kit(make_fixture_kit())

        self.assertEqual(tuple(pieces), PIECE_NAMES)
        for index, name in enumerate(PIECE_NAMES):
            color = (index * 23 + 20, index * 17 + 30, index * 13 + 40, 255)
            self.assertEqual(pieces[name].getpixel((18, 18)), color)

    def test_normalized_pieces_match_source_contracts_and_clear_edges(self) -> None:
        pieces = split_frame_kit(make_fixture_kit())

        for name, size in PIECE_SPECS.items():
            result = normalize_piece(pieces[name], size)
            self.assertEqual(result.size, size)
            self.assertEqual(max(edge_alpha(result)), 0)
            self.assertIsNone(result.getchannel("A").crop((0, 0, 4, size[1])).getbbox())

    def test_normalize_piece_removes_alpha_below_threshold(self) -> None:
        source = Image.new("RGBA", (12, 12), (0, 0, 0, 0))
        source.putpixel((5, 5), (10, 20, 30, 15))
        source.putpixel((6, 5), (40, 50, 60, 16))

        result = normalize_piece(source, (16, 16))

        self.assertEqual(result.getchannel("A").getbbox(), (4, 4, 12, 12))
        self.assertEqual(result.getpixel((4, 4)), (40, 50, 60, 16))

    def test_cli_writes_non_empty_rank_pieces_under_named_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "kit.png"
            output_root = root / "frames"
            make_fixture_kit(empty={"crest-top", "ornament-h", "ornament-v"}).save(source)

            completed = self.run_cli(
                "--family", "rank", "--name", "emerald", "--input", str(source),
                "--output-root", str(output_root),
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            written = output_root / "ranks" / "emerald"
            self.assertEqual(
                {path.stem for path in written.glob("*.png")}, set(BASE_PIECES)
            )
            with Image.open(written / "rail-h.png") as rail:
                self.assertEqual(rail.size, PIECE_SPECS["rail-h"])
                self.assertEqual(max(edge_alpha(rail)), 0)

    def test_cli_writes_prestige_to_numeric_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "kit.png"
            output_root = root / "frames"
            make_fixture_kit(empty={"ornament-h", "ornament-v"}).save(source)

            completed = self.run_cli(
                "--family", "prestige", "--name", "10", "--input", str(source),
                "--output-root", str(output_root),
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            written = output_root / "prestige" / "10"
            self.assertEqual(
                {path.stem for path in written.glob("*.png")}, set(BASE_PIECES) | {"crest-top"}
            )

    def test_cli_rejects_malformed_geometry_without_writing_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "malformed.png"
            output_root = root / "frames"
            Image.new("RGBA", (100, 99), (0, 0, 0, 0)).save(source)

            completed = self.run_cli(
                "--family", "rank", "--name", "iron", "--input", str(source),
                "--output-root", str(output_root),
            )

            self.assertNotEqual(completed.returncode, 0)
            self.assertIn("divisible by 3", completed.stderr)
            self.assertFalse(output_root.exists())

    def test_cli_rejects_empty_required_piece_without_writing_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "kit.png"
            output_root = root / "frames"
            make_fixture_kit(empty={"rail-v"}).save(source)

            completed = self.run_cli(
                "--family", "rank", "--name", "iron", "--input", str(source),
                "--output-root", str(output_root),
            )

            self.assertNotEqual(completed.returncode, 0)
            self.assertIn("required pieces are empty: rail-v", completed.stderr)
            self.assertFalse(output_root.exists())

    def test_cli_rejects_invalid_family_name_pair(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "kit.png"
            make_fixture_kit().save(source)

            completed = self.run_cli(
                "--family", "prestige", "--name", "eleven", "--input", str(source),
                "--output-root", str(root / "frames"),
            )

            self.assertNotEqual(completed.returncode, 0)
            self.assertIn("prestige name must be an integer from 1 through 10", completed.stderr)

    @staticmethod
    def run_cli(*arguments: str) -> subprocess.CompletedProcess[str]:
        script = Path(__file__).with_name("normalize_frame_art.py")
        return subprocess.run(
            [sys.executable, str(script), *arguments],
            check=False,
            capture_output=True,
            text=True,
        )


if __name__ == "__main__":
    unittest.main()
