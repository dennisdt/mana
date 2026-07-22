#!/usr/bin/env python3
"""Reject rank atlases that fall materially below the shipped mage art."""

from __future__ import annotations

import argparse
import json
import statistics
from dataclasses import asdict, dataclass
from pathlib import Path

from PIL import Image


ATLAS_SIZE = (448, 336)
CELL_SIZE = 112


@dataclass(frozen=True)
class CellMetrics:
    unique_colors: int
    visible_pixels: int
    outline_pixels: int
    boundary_pixels: int
    outline_coverage: float
    occupied_density: float


@dataclass(frozen=True)
class AtlasMetrics:
    cells: list[CellMetrics]
    median_unique_colors: float
    median_visible_pixels: float
    median_outline_coverage: float
    median_occupied_density: float


@dataclass(frozen=True)
class QualityReport:
    ok: bool
    failures: list[str]
    detail_ratio: float
    coverage_ratio: float
    outline_ratio: float
    density_ratio: float
    reference: AtlasMetrics
    candidate: AtlasMetrics


def atlas_metrics(image: Image.Image) -> AtlasMetrics:
    rgba = image.convert("RGBA")
    if rgba.size != ATLAS_SIZE:
        raise ValueError(f"atlas must be {ATLAS_SIZE[0]}x{ATLAS_SIZE[1]}, got {rgba.size}")

    cells: list[CellMetrics] = []
    for row in range(3):
        for column in range(4):
            cell = rgba.crop(
                (
                    column * CELL_SIZE,
                    row * CELL_SIZE,
                    (column + 1) * CELL_SIZE,
                    (row + 1) * CELL_SIZE,
                )
            )
            pixels = list(cell.get_flattened_data())
            visible_indexes = [index for index, pixel in enumerate(pixels) if pixel[3] > 16]
            visible = [pixels[index] for index in visible_indexes]

            xs = [index % CELL_SIZE for index in visible_indexes]
            ys = [index // CELL_SIZE for index in visible_indexes]
            if visible_indexes:
                occupied_area = (max(xs) - min(xs) + 1) * (max(ys) - min(ys) + 1)
                occupied_density = len(visible_indexes) / occupied_area
            else:
                occupied_density = 0.0

            boundary_indexes: list[int] = []
            for index in visible_indexes:
                x = index % CELL_SIZE
                y = index // CELL_SIZE
                neighbors = (
                    (x - 1, y),
                    (x + 1, y),
                    (x, y - 1),
                    (x, y + 1),
                )
                if any(
                    nx < 0
                    or nx >= CELL_SIZE
                    or ny < 0
                    or ny >= CELL_SIZE
                    or pixels[ny * CELL_SIZE + nx][3] <= 16
                    for nx, ny in neighbors
                ):
                    boundary_indexes.append(index)

            outline_pixels = sum(
                1
                for index in boundary_indexes
                if (
                    pixels[index][0] * 2126
                    + pixels[index][1] * 7152
                    + pixels[index][2] * 722
                )
                / 10_000
                < 96
            )
            outline_coverage = outline_pixels / max(len(boundary_indexes), 1)
            cells.append(
                CellMetrics(
                    unique_colors=len(set(visible)),
                    visible_pixels=len(visible),
                    outline_pixels=outline_pixels,
                    boundary_pixels=len(boundary_indexes),
                    outline_coverage=outline_coverage,
                    occupied_density=occupied_density,
                )
            )

    return AtlasMetrics(
        cells=cells,
        median_unique_colors=statistics.median(cell.unique_colors for cell in cells),
        median_visible_pixels=statistics.median(cell.visible_pixels for cell in cells),
        median_outline_coverage=statistics.median(cell.outline_coverage for cell in cells),
        median_occupied_density=statistics.median(cell.occupied_density for cell in cells),
    )


def compare_atlases(reference: Image.Image, candidate: Image.Image) -> QualityReport:
    reference_metrics = atlas_metrics(reference)
    candidate_metrics = atlas_metrics(candidate)
    detail_ratio = candidate_metrics.median_unique_colors / max(
        reference_metrics.median_unique_colors, 1
    )
    coverage_ratio = candidate_metrics.median_visible_pixels / max(
        reference_metrics.median_visible_pixels, 1
    )
    outline_ratio = candidate_metrics.median_outline_coverage / max(
        reference_metrics.median_outline_coverage, 0.001
    )
    density_ratio = candidate_metrics.median_occupied_density / max(
        reference_metrics.median_occupied_density, 0.001
    )

    failures: list[str] = []
    if detail_ratio < 0.12:
        failures.append("detail_palette")
    if not 0.45 <= coverage_ratio <= 1.85:
        failures.append("sprite_coverage")
    if outline_ratio < 0.45:
        failures.append("outline_coverage")
    if not 0.60 <= density_ratio <= 1.40:
        failures.append("occupied_density")

    return QualityReport(
        ok=not failures,
        failures=failures,
        detail_ratio=detail_ratio,
        coverage_ratio=coverage_ratio,
        outline_ratio=outline_ratio,
        density_ratio=density_ratio,
        reference=reference_metrics,
        candidate=candidate_metrics,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    args = parser.parse_args()

    with Image.open(args.reference) as reference, Image.open(args.candidate) as candidate:
        report = compare_atlases(reference, candidate)

    print(json.dumps(asdict(report), indent=2))
    return 0 if report.ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
