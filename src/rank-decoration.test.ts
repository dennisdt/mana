// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync } from "node:fs";
// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { inflateSync } from "node:zlib";
import { describe, expect, it } from "vitest";

type DecodedPng = {
  width: number;
  height: number;
  pixels: Uint8Array;
};

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

function visibleBounds(image: DecodedPng): [number, number, number, number] | null {
  let left = image.width;
  let top = image.height;
  let right = -1;
  let bottom = -1;

  for (let y = 0; y < image.height; y += 1) {
    for (let x = 0; x < image.width; x += 1) {
      if (image.pixels[(y * image.width + x) * 4 + 3] <= 16) continue;
      left = Math.min(left, x);
      top = Math.min(top, y);
      right = Math.max(right, x);
      bottom = Math.max(bottom, y);
    }
  }

  return right < 0 ? null : [left, top, right + 1, bottom + 1];
}

describe("rank decoration bitmaps", () => {
  it("locks every rank meter overlay to the HUD frame contract", () => {
    const tiers = [
      "naked", "plastic", "wood", "iron", "bronze", "silver", "gold",
      "platinum", "emerald", "diamond", "master", "legend", "champion", "godlike",
    ];

    for (const tier of tiers) {
      const image = decodeRgba(
        new URL(`../public/hud/mana-bar-frame-${tier}.png`, import.meta.url),
      );
      expect([image.width, image.height], tier).toEqual([288, 40]);
      expect(edgeAlpha(image), tier).toBe(0);
      expect(visiblePixels(image), tier).toBeGreaterThan(1_200);
      expect(visibleBounds(image), tier).toEqual([2, 2, 286, 38]);
    }
  });

  it("locks every prestige medallion to the badge contract", () => {
    for (let prestige = 1; prestige <= 10; prestige += 1) {
      const image = decodeRgba(
        new URL(`../public/badges/prestige-${prestige}.png`, import.meta.url),
      );
      expect([image.width, image.height], String(prestige)).toEqual([96, 96]);
      expect(edgeAlpha(image), String(prestige)).toBe(0);
      expect(visiblePixels(image), String(prestige)).toBeGreaterThan(1_000);
    }
  });
});
