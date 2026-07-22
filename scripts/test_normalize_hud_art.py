#!/usr/bin/env python3
"""Focused tests for HUD-art normalization."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))

from normalize_hud_art import normalize_badge, normalize_meter


def edge_alpha(image: Image.Image) -> list[int]:
    alpha = image.getchannel("A")
    width, height = image.size
    return (
        [alpha.getpixel((x, 0)) for x in range(width)]
        + [alpha.getpixel((x, height - 1)) for x in range(width)]
        + [alpha.getpixel((0, y)) for y in range(height)]
        + [alpha.getpixel((width - 1, y)) for y in range(height)]
    )


class HudArtNormalizationTests(unittest.TestCase):
    def test_meter_normalization_preserves_wide_art_inside_clear_edges(self) -> None:
        source = Image.new("RGBA", (900, 300), (0, 0, 0, 0))
        ImageDraw.Draw(source).rounded_rectangle(
            (40, 80, 860, 220), 30, fill=(220, 180, 80, 255)
        )

        result = normalize_meter(source)

        self.assertEqual(result.size, (288, 40))
        self.assertEqual(max(edge_alpha(result)), 0)

    def test_meter_normalization_fills_fixed_bounds_for_narrow_art(self) -> None:
        source = Image.new("RGBA", (300, 700), (0, 0, 0, 0))
        ImageDraw.Draw(source).rectangle(
            (100, 50, 200, 650), fill=(220, 180, 80, 255)
        )

        result = normalize_meter(source)

        visible_bounds = result.getchannel("A").point(
            lambda alpha: 255 if alpha > 16 else 0
        ).getbbox()
        self.assertEqual(visible_bounds, (2, 2, 286, 38))

    def test_badge_normalization_centers_crest_inside_clear_edges(self) -> None:
        source = Image.new("RGBA", (700, 700), (0, 0, 0, 0))
        ImageDraw.Draw(source).ellipse(
            (80, 40, 620, 650), fill=(220, 180, 80, 255)
        )

        result = normalize_badge(source)

        self.assertEqual(result.size, (96, 96))
        self.assertEqual(max(edge_alpha(result)), 0)


if __name__ == "__main__":
    unittest.main()
