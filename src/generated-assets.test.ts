// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync, readdirSync } from "node:fs";
// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { inflateSync } from "node:zlib";
import { describe, expect, it } from "vitest";

type DecodedPng = {
  width: number;
  height: number;
  pixels: Uint8Array;
};

const PIECE_SPECS = {
  "corner-tl": [96, 96],
  "rail-h": [128, 32],
  "corner-tr": [96, 96],
  "rail-v": [32, 128],
  "crest-top": [192, 96],
  "ornament-h": [64, 32],
  "corner-bl": [96, 96],
  "ornament-v": [32, 64],
  "corner-br": [96, 96],
} as const;

const BASE_PIECES = [
  "corner-tl", "rail-h", "corner-tr", "rail-v", "corner-bl", "corner-br",
] as const;

const RANK_EXTRAS = {
  gold: ["crest-top"],
  platinum: ["crest-top"],
  emerald: ["crest-top", "ornament-h", "ornament-v"],
  diamond: ["crest-top", "ornament-h", "ornament-v"],
  master: ["crest-top", "ornament-h", "ornament-v"],
  legend: ["crest-top", "ornament-h", "ornament-v"],
  champion: ["crest-top", "ornament-h", "ornament-v"],
  godlike: ["crest-top", "ornament-h", "ornament-v"],
} as const;

const RANKS = [
  "plastic", "wood", "iron", "bronze", "silver", "gold", "platinum",
  "emerald", "diamond", "master", "legend", "champion", "godlike",
] as const;

type PieceName = keyof typeof PIECE_SPECS;

function uint32(bytes: Uint8Array, offset: number): number {
  return new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(offset);
}

function concatBytes(parts: Uint8Array[]): Uint8Array {
  const result = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.length;
  }
  return result;
}

function paeth(left: number, up: number, upperLeft: number): number {
  const estimate = left + up - upperLeft;
  const leftDistance = Math.abs(estimate - left);
  const upDistance = Math.abs(estimate - up);
  const upperLeftDistance = Math.abs(estimate - upperLeft);
  if (leftDistance <= upDistance && leftDistance <= upperLeftDistance) return left;
  return upDistance <= upperLeftDistance ? up : upperLeft;
}

function decodeRgba(url: URL): DecodedPng {
  const bytes = new Uint8Array(readFileSync(url));
  expect(Array.from(bytes.subarray(0, 8))).toEqual([137, 80, 78, 71, 13, 10, 26, 10]);
  const width = uint32(bytes, 16);
  const height = uint32(bytes, 20);
  expect(bytes[24]).toBe(8);
  expect(bytes[25]).toBe(6);
  expect(bytes[28]).toBe(0);

  const idat: Uint8Array[] = [];
  for (let cursor = 8; cursor < bytes.length; ) {
    const length = uint32(bytes, cursor);
    const type = String.fromCharCode(...bytes.subarray(cursor + 4, cursor + 8));
    if (type === "IDAT") idat.push(bytes.slice(cursor + 8, cursor + 8 + length));
    cursor += length + 12;
  }

  const filtered = new Uint8Array(inflateSync(concatBytes(idat)));
  const stride = width * 4;
  const pixels = new Uint8Array(stride * height);
  for (let y = 0; y < height; y += 1) {
    const sourceStart = y * (stride + 1);
    const filter = filtered[sourceStart];
    expect(filter).toBeGreaterThanOrEqual(0);
    expect(filter).toBeLessThanOrEqual(4);
    for (let x = 0; x < stride; x += 1) {
      const raw = filtered[sourceStart + 1 + x];
      const target = y * stride + x;
      const left = x >= 4 ? pixels[target - 4] : 0;
      const up = y > 0 ? pixels[target - stride] : 0;
      const upperLeft = y > 0 && x >= 4 ? pixels[target - stride - 4] : 0;
      const value =
        filter === 0 ? raw :
        filter === 1 ? raw + left :
        filter === 2 ? raw + up :
        filter === 3 ? raw + Math.floor((left + up) / 2) :
        raw + paeth(left, up, upperLeft);
      pixels[target] = value & 0xff;
    }
  }
  return { width, height, pixels };
}

function edgeAlpha(image: DecodedPng): number {
  let maximum = 0;
  for (let y = 0; y < image.height; y += 1) {
    for (let x = 0; x < image.width; x += 1) {
      if (x !== 0 && x !== image.width - 1 && y !== 0 && y !== image.height - 1) continue;
      maximum = Math.max(maximum, image.pixels[(y * image.width + x) * 4 + 3]);
    }
  }
  return maximum;
}

function visiblePixels(image: DecodedPng): number {
  let visible = 0;
  for (let index = 3; index < image.pixels.length; index += 4) {
    if (image.pixels[index] > 16) visible += 1;
  }
  return visible;
}

function visibleBounds(image: DecodedPng): {
  width: number;
  height: number;
} {
  let minX = image.width;
  let minY = image.height;
  let maxX = -1;
  let maxY = -1;
  for (let y = 0; y < image.height; y += 1) {
    for (let x = 0; x < image.width; x += 1) {
      if (image.pixels[(y * image.width + x) * 4 + 3] <= 16) continue;
      minX = Math.min(minX, x);
      minY = Math.min(minY, y);
      maxX = Math.max(maxX, x);
      maxY = Math.max(maxY, y);
    }
  }
  return {
    width: maxX >= minX ? maxX - minX + 1 : 0,
    height: maxY >= minY ? maxY - minY + 1 : 0,
  };
}

function assertKit(directory: URL, expectedPieces: readonly PieceName[]): void {
  const filenames = readdirSync(directory)
    .filter((name: string) => name.endsWith(".png"))
    .sort();
  expect(filenames).toEqual(expectedPieces.map((name) => `${name}.png`).sort());

  for (const piece of expectedPieces) {
    const image = decodeRgba(new URL(`${piece}.png`, directory));
    expect([image.width, image.height], piece).toEqual(PIECE_SPECS[piece]);
    expect(edgeAlpha(image), piece).toBe(0);
    expect(visiblePixels(image), piece).toBeGreaterThan(image.width * image.height * 0.02);
    const bounds = visibleBounds(image);
    if (piece === "rail-h") {
      expect(
        bounds.width,
        `${directory.pathname}${piece}.png repeat span`,
      ).toBeGreaterThanOrEqual(image.width * 0.65);
    }
    if (piece === "rail-v") {
      expect(
        bounds.height,
        `${directory.pathname}${piece}.png repeat span`,
      ).toBeGreaterThanOrEqual(image.height * 0.65);
    }
  }
}

describe("generated frame assets", () => {
  it("locks every illustrated rank to its exact piece contract", () => {
    for (const rank of RANKS) {
      const extras = rank in RANK_EXTRAS
        ? RANK_EXTRAS[rank as keyof typeof RANK_EXTRAS]
        : [];
      assertKit(
        new URL(`../public/frames/ranks/${rank}/`, import.meta.url),
        [...BASE_PIECES, ...extras],
      );
    }
  });

  it("locks every prestige overlay to its seven-piece contract", () => {
    const prestigePieces = [...BASE_PIECES, "crest-top"] as const;
    for (let prestige = 1; prestige <= 10; prestige += 1) {
      assertKit(
        new URL(`../public/frames/prestige/${prestige}/`, import.meta.url),
        prestigePieces,
      );
    }
  });
});
