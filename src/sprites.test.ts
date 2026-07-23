// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { existsSync, readFileSync, readdirSync } from "node:fs";
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

function verifyAtlas(relativePath: string, maxBaselineSpread = 12): void {
  const image = decodeRgba(new URL(relativePath, import.meta.url));
  expect(image.width % 448).toBe(0);
  const retinaScale = image.width / 448;
  expect([1, 2]).toContain(retinaScale);
  expect(image.height).toBe(336 * retinaScale);
  const cell = 112 * retinaScale;
  const margin = 4 * retinaScale;
  const alphaAt = (x: number, y: number) => image.pixels[(y * image.width + x) * 4 + 3];

  for (let row = 0; row < 3; row += 1) {
    const bottoms: number[] = [];
    for (let column = 0; column < 4; column += 1) {
      let edgeAlpha = 0;
      let visible = 0;
      let bottom = -1;
      for (let y = 0; y < cell; y += 1) {
        for (let x = 0; x < cell; x += 1) {
          const alpha = alphaAt(column * cell + x, row * cell + y);
          if (x < margin || x >= cell - margin || y < margin || y >= cell - margin) {
            edgeAlpha = Math.max(edgeAlpha, alpha);
          }
          if (alpha > 16) {
            visible += 1;
            bottom = Math.max(bottom, y);
          }
        }
      }
      expect(edgeAlpha).toBe(0);
      expect(visible).toBeGreaterThan(600 * retinaScale * retinaScale);
      expect(visible).toBeLessThan(10_500 * retinaScale * retinaScale);
      expect(bottom).toBeGreaterThan(60 * retinaScale);
      bottoms.push(bottom);
    }
    expect(Math.max(...bottoms) - Math.min(...bottoms)).toBeLessThanOrEqual(
      maxBaselineSpread * retinaScale,
    );
  }
}

describe("elemental mage sprite atlases", () => {
  it("keeps the Codex atlas aligned, padded, and transparent", () => {
    verifyAtlas("../public/sprites/codex-ice-lightning.png");
  });

  it("keeps the Claude atlas aligned, padded, and transparent", () => {
    verifyAtlas("../public/sprites/claude-fire-poison.png");
  });

  it("validates every per-rank sheet the moment its art lands", () => {
    // Rank sheets are Codex-generated later; an empty glob passes so the
    // suite gates each drop automatically without blocking on missing art.
    const names: string[] = readdirSync(new URL("../public/sprites/", import.meta.url));
    for (const name of names.filter((file) => file.includes("-rank-"))) {
      // Codex Master working frame four intentionally drops its shield effect
      // lower than the character. Detached dots must not be used to fake an
      // aligned baseline in the other frames.
      const maxBaselineSpread = name === "codex-rank-master.png" ? 18 : 12;
      verifyAtlas(`../public/sprites/${name}`, maxBaselineSpread);
    }
  });

  it("keeps the Codex Champion upgrade at true four-times Retina resolution", () => {
    const champion = decodeRgba(
      new URL("../public/sprites/codex-rank-champion.png", import.meta.url),
    );
    expect([champion.width, champion.height]).toEqual([896, 672]);
  });

  it("keeps Claude Master working frames free of left-edge debris", () => {
    const image = decodeRgba(
      new URL("../public/sprites/claude-rank-master.png", import.meta.url),
    );
    expect([image.width, image.height]).toEqual([448, 336]);
    const cell = 112;
    const workingTop = cell;
    const workingBottom = cell * 2;
    for (let column = 0; column < 4; column += 1) {
      for (let y = workingTop; y < workingBottom; y += 1) {
        for (let x = column * cell; x < column * cell + 12; x += 1) {
          expect(image.pixels[(y * image.width + x) * 4 + 3]).toBeLessThanOrEqual(16);
        }
      }
    }
  });

  it("keeps Codex Master working frames free of detached bottom dots", () => {
    const image = decodeRgba(
      new URL("../public/sprites/codex-rank-master.png", import.meta.url),
    );
    expect([image.width, image.height]).toEqual([448, 336]);
    const cell = 112;
    const workingTop = cell;
    for (let column = 0; column < 3; column += 1) {
      for (let y = workingTop + 100; y < workingTop + cell; y += 1) {
        for (let x = column * cell; x < (column + 1) * cell; x += 1) {
          expect(image.pixels[(y * image.width + x) * 4 + 3]).toBeLessThanOrEqual(16);
        }
      }
    }
  });
});

it("retires the deterministic pixel familiar assets", () => {
  expect(existsSync(new URL("../public/sprites/clawd.png", import.meta.url))).toBe(false);
  expect(existsSync(new URL("../public/sprites/nimbus.png", import.meta.url))).toBe(false);
  expect(existsSync(new URL("../scripts/gen-sprites.py", import.meta.url))).toBe(false);
});
