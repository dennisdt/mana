#!/usr/bin/env python3
"""Focused tests for generated frame-kit normalization."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))

from normalize_frame_art import (
    PIECE_SPECS,
    main,
    make_repeatable_rail,
    normalize_piece,
    split_frame_kit,
)


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


def assert_repeatable_rail(test: unittest.TestCase, image: Image.Image, axis: str) -> None:
    rgba = image.convert("RGBA")
    alpha = rgba.getchannel("A")
    width, height = rgba.size

    if axis == "horizontal":
        test.assertTrue(
            all(alpha.crop((x, 0, x + 1, height)).getbbox() is not None for x in range(width))
        )
        test.assertEqual(
            [rgba.getpixel((0, y)) for y in range(height)],
            [rgba.getpixel((width - 1, y)) for y in range(height)],
        )
        test.assertIsNone(alpha.crop((0, 0, width, 4)).getbbox())
        test.assertIsNone(alpha.crop((0, height - 4, width, height)).getbbox())
        return

    test.assertTrue(
        all(alpha.crop((0, y, width, y + 1)).getbbox() is not None for y in range(height))
    )
    test.assertEqual(
        [rgba.getpixel((x, 0)) for x in range(width)],
        [rgba.getpixel((x, height - 1)) for x in range(width)],
    )
    test.assertIsNone(alpha.crop((0, 0, 4, height)).getbbox())
    test.assertIsNone(alpha.crop((width - 4, 0, width, height)).getbbox())


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


def make_asymmetric_piece() -> Image.Image:
    image = Image.new("RGBA", (30, 30), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    draw.polygon(((5, 9), (13, 9), (10, 11), (13, 13), (5, 13)), fill=(20, 40, 60, 255))
    return image


def directory_bytes(directory: Path) -> dict[str, bytes]:
    return {
        path.relative_to(directory).as_posix(): path.read_bytes()
        for path in sorted(directory.rglob("*"))
        if path.is_file()
    }


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

    def test_normalize_piece_preserves_aspect_ratio_and_centers_asymmetric_art(self) -> None:
        result = normalize_piece(make_asymmetric_piece(), (32, 24))
        alpha = result.getchannel("A")

        self.assertIsNone(alpha.crop((0, 0, 4, 24)).getbbox())
        self.assertIsNone(alpha.crop((0, 0, 32, 4)).getbbox())
        self.assertIsNone(alpha.crop((28, 0, 32, 24)).getbbox())
        self.assertIsNone(alpha.crop((0, 20, 32, 24)).getbbox())
        self.assertEqual(alpha.getbbox(), (4, 5, 28, 18))
        visible_width, visible_height = 24, 13
        self.assertAlmostEqual(visible_width / visible_height, 9 / 5, delta=0.1)
        self.assertLessEqual(abs(4 - (32 - 28)), 1)
        self.assertLessEqual(abs(5 - (24 - 18)), 1)

    def test_normalize_piece_removes_alpha_below_threshold(self) -> None:
        source = Image.new("RGBA", (12, 12), (0, 0, 0, 0))
        source.putpixel((5, 5), (10, 20, 30, 15))
        source.putpixel((6, 5), (40, 50, 60, 16))

        result = normalize_piece(source, (16, 16))

        self.assertEqual(result.getchannel("A").getbbox(), (4, 4, 12, 12))
        self.assertEqual(result.getpixel((4, 4)), (40, 50, 60, 16))

    def test_repeatable_rails_use_an_exact_central_crop_and_mirror(self) -> None:
        horizontal = Image.new("RGBA", (128, 32), (0, 0, 0, 0))
        for x in range(32, 96):
            for y in range(4, 28):
                horizontal.putpixel((x, y), (x, y, 255 - x, 255))

        horizontal_result = make_repeatable_rail(horizontal, horizontal=True)
        horizontal_segment = horizontal.crop((32, 0, 96, 32))
        self.assertEqual(
            horizontal_result.crop((0, 0, 64, 32)).tobytes(),
            horizontal_segment.tobytes(),
        )
        self.assertEqual(
            horizontal_result.crop((64, 0, 128, 32)).tobytes(),
            horizontal_segment.transpose(Image.Transpose.FLIP_LEFT_RIGHT).tobytes(),
        )
        assert_repeatable_rail(self, horizontal_result, "horizontal")

        vertical = Image.new("RGBA", (32, 128), (0, 0, 0, 0))
        for y in range(32, 96):
            for x in range(4, 28):
                vertical.putpixel((x, y), (x, y, 255 - y, 255))

        vertical_result = make_repeatable_rail(vertical, horizontal=False)
        vertical_segment = vertical.crop((0, 32, 32, 96))
        self.assertEqual(
            vertical_result.crop((0, 0, 32, 64)).tobytes(),
            vertical_segment.tobytes(),
        )
        self.assertEqual(
            vertical_result.crop((0, 64, 32, 128)).tobytes(),
            vertical_segment.transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes(),
        )
        assert_repeatable_rail(self, vertical_result, "vertical")

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
                assert_repeatable_rail(self, rail, "horizontal")
            with Image.open(written / "rail-v.png") as rail:
                self.assertEqual(rail.size, PIECE_SPECS["rail-v"])
                assert_repeatable_rail(self, rail, "vertical")

    def test_cli_successful_publication_removes_stale_optional_pieces(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "kit.png"
            output_root = root / "frames"
            destination = output_root / "ranks" / "emerald"
            destination.mkdir(parents=True)
            (destination / "ornament-h.png").write_bytes(b"stale optional piece")
            make_fixture_kit(empty={"crest-top", "ornament-h", "ornament-v"}).save(source)

            completed = self.run_cli(
                "--family", "rank", "--name", "emerald", "--input", str(source),
                "--output-root", str(output_root),
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertFalse((destination / "ornament-h.png").exists())
            self.assertEqual(
                {path.stem for path in destination.glob("*.png")}, set(BASE_PIECES)
            )

    def test_cli_late_save_failure_preserves_previous_complete_destination(self) -> None:
        with tempfile.TemporaryDirectory() as temp_directory:
            root = Path(temp_directory)
            source = root / "kit.png"
            output_root = root / "frames"
            destination = output_root / "ranks" / "iron"
            destination.mkdir(parents=True)
            (destination / "corner-tl.png").write_bytes(b"previous corner")
            (destination / "ornament-h.png").write_bytes(b"previous optional")
            (destination / "previous-only.txt").write_bytes(b"previous complete kit marker")
            before = directory_bytes(destination)
            make_fixture_kit(empty={"crest-top", "ornament-h", "ornament-v"}).save(source)

            original_save = Image.Image.save

            def fail_on_late_piece(image: Image.Image, fp: str | Path, *args: object, **kwargs: object) -> None:
                if Path(fp).name == "rail-h.png":
                    raise OSError("simulated late save failure")
                original_save(image, fp, *args, **kwargs)

            with mock.patch.object(Image.Image, "save", new=fail_on_late_piece):
                with self.assertRaises(SystemExit) as error:
                    main(
                        [
                            "--family", "rank", "--name", "iron", "--input", str(source),
                            "--output-root", str(output_root),
                        ]
                    )

            self.assertEqual(error.exception.code, 2)
            self.assertEqual(directory_bytes(destination), before)
            self.assertEqual(
                {path.name for path in destination.iterdir()}, set(before)
            )
            self.assertEqual(
                list(destination.parent.glob(f".{destination.name}.*")), []
            )

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
