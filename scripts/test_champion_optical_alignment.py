#!/usr/bin/env python3
"""Regression tests for Claude/Codex Champion optical alignment."""

from __future__ import annotations

import hashlib
from pathlib import Path
import unittest

from PIL import Image

from align_codex_champion import align_champion


ROOT = Path(__file__).resolve().parents[1]
BASE_ATLAS_SIZE = (448, 336)
BASE_CELL_SIZE = 112
BASE_CELL_MARGIN = 4
ALPHA_THRESHOLD = 16
MAX_PROVIDER_DELTA = 4
EXPECTED_WORKING_ROW_SHA256 = "f39b9deacbf5da357cea0f26a7f1a7505cb7d683117b8a93345cdebc875ca93b"


def retina_scale(atlas: Image.Image) -> int:
    if atlas.width % BASE_ATLAS_SIZE[0] != 0:
        raise AssertionError(f"invalid Champion atlas width: {atlas.width}")
    scale = atlas.width // BASE_ATLAS_SIZE[0]
    if scale not in (1, 2) or atlas.height != BASE_ATLAS_SIZE[1] * scale:
        raise AssertionError(f"invalid Champion atlas size: {atlas.size}")
    return scale


def visible_bbox(cell: Image.Image) -> tuple[int, int, int, int]:
    bounds = cell.getchannel("A").point(
        lambda alpha: 255 if alpha > ALPHA_THRESHOLD else 0
    ).getbbox()
    if bounds is None:
        raise AssertionError("Champion cell has no visible pixels")
    return bounds


def row_metrics(atlas: Image.Image, row: int) -> list[tuple[float, float]]:
    scale = retina_scale(atlas)
    cell_size = BASE_CELL_SIZE * scale
    metrics: list[tuple[int, int]] = []
    for column in range(4):
        cell = atlas.crop(
            (
                column * cell_size,
                row * cell_size,
                (column + 1) * cell_size,
                (row + 1) * cell_size,
            )
        )
        left, top, right, bottom = visible_bbox(cell)
        del left, right
        metrics.append(((bottom - top) / scale, bottom / scale))
    return metrics


def assert_cell_safety(test: unittest.TestCase, atlas: Image.Image) -> None:
    scale = retina_scale(atlas)
    cell_size = BASE_CELL_SIZE * scale
    margin = BASE_CELL_MARGIN * scale
    test.assertEqual(atlas.mode, "RGBA")
    alpha = atlas.getchannel("A")
    for row in range(3):
        for column in range(4):
            for y in range(cell_size):
                for x in range(cell_size):
                    if margin <= x < cell_size - margin and margin <= y < cell_size - margin:
                        continue
                    test.assertEqual(
                        alpha.getpixel((column * cell_size + x, row * cell_size + y)),
                        0,
                        f"nontransparent safety margin in row {row}, column {column}",
                    )


class ChampionOpticalAlignmentTests(unittest.TestCase):
    def test_idle_and_hover_match_provider_height_and_baseline(self) -> None:
        with Image.open(
            ROOT / "public/sprites/claude-rank-champion.png"
        ) as claude_source, Image.open(
            ROOT / "public/sprites/codex-rank-champion.png"
        ) as codex_source:
            claude = claude_source.convert("RGBA")
            codex = codex_source.convert("RGBA")

        assert_cell_safety(self, claude)
        assert_cell_safety(self, codex)

        for row_name, row in (("idle", 0), ("hover", 2)):
            claude_metrics = row_metrics(claude, row)
            codex_metrics = row_metrics(codex, row)
            for column, ((claude_height, claude_bottom), (codex_height, codex_bottom)) in enumerate(
                zip(claude_metrics, codex_metrics, strict=True)
            ):
                self.assertLessEqual(
                    abs(claude_height - codex_height),
                    MAX_PROVIDER_DELTA,
                    f"{row_name} column {column} height mismatch: "
                    f"Claude={claude_height}, Codex={codex_height}",
                )
                self.assertLessEqual(
                    abs(claude_bottom - codex_bottom),
                    MAX_PROVIDER_DELTA,
                    f"{row_name} column {column} baseline mismatch: "
                    f"Claude={claude_bottom}, Codex={codex_bottom}",
                )

    def test_alignment_is_idempotent_and_preserves_accepted_working_row(self) -> None:
        with Image.open(
            ROOT / "public/sprites/codex-rank-champion.png"
        ) as source:
            atlas = source.convert("RGBA")

        cell_size = BASE_CELL_SIZE * retina_scale(atlas)
        working_row = atlas.crop((0, cell_size, atlas.width, cell_size * 2)).tobytes()
        self.assertEqual(
            hashlib.sha256(working_row).hexdigest(),
            EXPECTED_WORKING_ROW_SHA256,
        )

        first_pass = align_champion(atlas)
        second_pass = align_champion(first_pass)
        self.assertEqual(second_pass.tobytes(), first_pass.tobytes())
        self.assertEqual(
            hashlib.sha256(
                first_pass.crop((0, cell_size, atlas.width, cell_size * 2)).tobytes()
            ).hexdigest(),
            EXPECTED_WORKING_ROW_SHA256,
        )


if __name__ == "__main__":
    unittest.main()
