#!/usr/bin/env python3
"""Focused tests for the rank-art quality gate."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))

from rank_art_quality import atlas_metrics, compare_atlases
from normalize_rank_atlas import normalize_atlas


def atlas_with_palette(color_count: int) -> Image.Image:
    atlas = Image.new("RGBA", (448, 336), (0, 0, 0, 0))
    for row in range(3):
        for column in range(4):
            left = column * 112 + 20
            top = row * 112 + 18
            for index in range(72 * 76):
                x = left + index % 72
                y = top + index // 72
                shade = index % color_count
                atlas.putpixel(
                    (x, y),
                    (shade % 256, (shade // 256) * 127, (shade * 53) % 251, 255),
                )
    return atlas


def atlas_with_outline(*, dark_outline: bool = True, sparse: bool = False) -> Image.Image:
    atlas = Image.new("RGBA", (448, 336), (0, 0, 0, 0))
    for row in range(3):
        for column in range(4):
            left = column * 112 + 20
            top = row * 112 + 18
            for y in range(76):
                for x in range(72):
                    edge = x < 3 or x >= 69 or y < 3 or y >= 73
                    if sparse and (x + y) % 3:
                        continue
                    color = (
                        (18, 22, 31, 255)
                        if edge and dark_outline
                        else (150 + (x * 3) % 90, 120 + (y * 5) % 100, 90 + (x + y) % 120, 255)
                    )
                    atlas.putpixel((left + x, top + y), color)
    return atlas


class RankArtQualityTests(unittest.TestCase):
    def test_normalizer_builds_exact_cells_with_clear_four_pixel_margins(self) -> None:
        source = Image.new("RGBA", (1000, 750), (0, 0, 0, 0))
        for row in range(3):
            for column in range(4):
                left = round(column * source.width / 4)
                right = round((column + 1) * source.width / 4)
                top = round(row * source.height / 3)
                bottom = round((row + 1) * source.height / 3)
                for y in range(top, bottom):
                    for x in range(left, right):
                        source.putpixel((x, y), (255, 170, 40, 255))

        normalized = normalize_atlas(source)

        self.assertEqual(normalized.size, (448, 336))
        for row in range(3):
            for column in range(4):
                for y in range(row * 112, (row + 1) * 112):
                    for x in range(column * 112, (column + 1) * 112):
                        if (
                            x - column * 112 < 4
                            or (column + 1) * 112 - 1 - x < 4
                            or y - row * 112 < 4
                            or (row + 1) * 112 - 1 - y < 4
                        ):
                            self.assertEqual(normalized.getpixel((x, y))[3], 0)

    def test_normalizer_aligns_animation_baselines_within_each_row(self) -> None:
        source = Image.new("RGBA", (1000, 750), (0, 0, 0, 0))
        source_cell_width = source.width // 4
        source_cell_height = source.height // 3
        for row in range(3):
            for column in range(4):
                top = row * source_cell_height + 20
                bottom = (row + 1) * source_cell_height - 20 - column * 25
                for y in range(top, bottom):
                    for x in range(column * source_cell_width + 60, column * source_cell_width + 190):
                        source.putpixel((x, y), (255, 170, 40, 255))
                faint_y = min(bottom + 20, (row + 1) * source_cell_height - 1)
                for x in range(column * source_cell_width + 60, column * source_cell_width + 190):
                    source.putpixel((x, faint_y), (255, 170, 40, 8))

        normalized = normalize_atlas(source)

        for row in range(3):
            bottoms = []
            for column in range(4):
                cell = normalized.crop(
                    (column * 112, row * 112, (column + 1) * 112, (row + 1) * 112)
                )
                bbox = cell.getchannel("A").point(lambda alpha: 255 if alpha > 16 else 0).getbbox()
                self.assertIsNotNone(bbox)
                bottoms.append(bbox[3] - 1)
            self.assertLessEqual(max(bottoms) - min(bottoms), 12)
            for column in range(4):
                cell = normalized.crop(
                    (column * 112, row * 112, (column + 1) * 112, (row + 1) * 112)
                )
                alpha = cell.getchannel("A")
                edge_values = []
                for y in range(112):
                    for x in range(112):
                        if x < 4 or x >= 108 or y < 4 or y >= 108:
                            edge_values.append(alpha.getpixel((x, y)))
                self.assertEqual(max(edge_values), 0)

    def test_normalizer_can_emit_four_times_retina_cells_with_scaled_margins(self) -> None:
        source = Image.new("RGBA", (1200, 900), (0, 0, 0, 0))
        for row in range(3):
            for column in range(4):
                left = column * 300 + 50
                top = row * 300 + 45
                for y in range(top, top + 210):
                    for x in range(left, left + 200):
                        source.putpixel((x, y), (34, 152, 255, 255))

        normalized = normalize_atlas(source, cell_size=224, cell_margin=8)

        self.assertEqual(normalized.size, (896, 672))
        for row in range(3):
            for column in range(4):
                cell = normalized.crop(
                    (column * 224, row * 224, (column + 1) * 224, (row + 1) * 224)
                )
                alpha = cell.getchannel("A")
                edge_values = []
                for y in range(224):
                    for x in range(224):
                        if x < 8 or x >= 216 or y < 8 or y >= 216:
                            edge_values.append(alpha.getpixel((x, y)))
                self.assertEqual(max(edge_values), 0)

    def test_normalizer_accepts_overlapping_source_rows_for_tall_ornaments(self) -> None:
        source = Image.new("RGBA", (400, 300), (0, 0, 0, 0))
        for column in range(4):
            left = column * 100 + 20
            for y in range(18, 82):
                for x in range(left, left + 60):
                    source.putpixel((x, y), (255, 210, 64, 255))
            for y in range(115, 178):
                for x in range(left, left + 60):
                    source.putpixel((x, y), (80, 170, 255, 255))
            # The final row's staff tip begins above the equal 200px row
            # boundary, while its body continues well below it.
            for y in range(184, 278):
                for x in range(left, left + 60):
                    source.putpixel((x, y), (255, 145, 32, 255))

        normalized = normalize_atlas(
            source,
            source_row_bounds=[(0, 95), (105, 190), (180, 290)],
        )

        third_row = normalized.crop((0, 224, 112, 336)).getchannel("A")
        self.assertIsNotNone(third_row.getbbox())
        self.assertGreater(
            sum(value > 16 for value in third_row.get_flattened_data()),
            3_000,
        )

    def test_metrics_measure_every_animation_cell(self) -> None:
        metrics = atlas_metrics(atlas_with_palette(512))

        self.assertEqual(len(metrics.cells), 12)
        self.assertGreater(metrics.median_unique_colors, 400)
        self.assertGreater(metrics.median_visible_pixels, 5_000)
        self.assertGreater(metrics.median_occupied_density, 0.95)

    def test_metrics_compare_two_times_and_four_times_retina_atlases(self) -> None:
        reference = atlas_with_palette(512)
        retina_candidate = reference.resize((896, 672), Image.Resampling.NEAREST)

        report = compare_atlases(reference, retina_candidate)

        self.assertTrue(report.ok)
        self.assertAlmostEqual(report.coverage_ratio, 1.0)
        self.assertAlmostEqual(report.density_ratio, 1.0)

    def test_metrics_measure_dark_outline_coverage(self) -> None:
        outlined = atlas_metrics(atlas_with_outline(dark_outline=True))
        unoutlined = atlas_metrics(atlas_with_outline(dark_outline=False))

        self.assertGreater(outlined.median_outline_coverage, 0.9)
        self.assertLess(unoutlined.median_outline_coverage, 0.1)

    def test_flat_recolor_fails_against_illustrated_reference(self) -> None:
        reference = atlas_with_palette(512)
        flat_candidate = atlas_with_palette(12)

        report = compare_atlases(reference, flat_candidate)

        self.assertFalse(report.ok)
        self.assertIn("detail_palette", report.failures)

    def test_comparably_detailed_candidate_passes(self) -> None:
        reference = atlas_with_palette(512)
        candidate = atlas_with_palette(384)

        report = compare_atlases(reference, candidate)

        self.assertTrue(report.ok)
        self.assertEqual(report.failures, [])

    def test_candidate_without_reference_like_outline_fails(self) -> None:
        reference = atlas_with_outline(dark_outline=True)
        candidate = atlas_with_outline(dark_outline=False)

        report = compare_atlases(reference, candidate)

        self.assertFalse(report.ok)
        self.assertIn("outline_coverage", report.failures)

    def test_candidate_with_sparse_occupancy_fails(self) -> None:
        reference = atlas_with_outline(dark_outline=True)
        candidate = atlas_with_outline(dark_outline=True, sparse=True)

        report = compare_atlases(reference, candidate)

        self.assertFalse(report.ok)
        self.assertIn("occupied_density", report.failures)


if __name__ == "__main__":
    unittest.main()
