import { afterEach, describe, expect, it, vi } from "vitest";
import { defaultSheet, probeImage, rankSheetUrl, resolveSheet } from "./cosmetics";

function stubImage(loads: (url: string) => boolean): void {
  class StubImage {
    onload: (() => void) | null = null;
    onerror: (() => void) | null = null;
    set src(url: string) {
      queueMicrotask(() => (loads(url) ? this.onload?.() : this.onerror?.()));
    }
  }
  vi.stubGlobal("Image", StubImage);
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("rank sprite sheets", () => {
  it("names rank sheets by provider and tier", () => {
    expect(rankSheetUrl("claude", "silver")).toBe("/sprites/claude-rank-silver.png");
    expect(rankSheetUrl("codex", "godlike")).toBe("/sprites/codex-rank-godlike.png");
  });

  it("keeps the shipped atlases as defaults", () => {
    expect(defaultSheet("claude")).toBe("/sprites/claude-fire-poison.png");
    expect(defaultSheet("codex")).toBe("/sprites/codex-ice-lightning.png");
  });

  it("probes an image by load or error", async () => {
    stubImage((url) => url.endsWith("good.png"));
    await expect(probeImage("/sprites/good.png")).resolves.toBe(true);
    await expect(probeImage("/sprites/bad.png")).resolves.toBe(false);
  });

  it("resolves the rank sheet when its art exists", async () => {
    stubImage((url) => url.includes("-rank-"));
    await expect(resolveSheet("codex", "gold")).resolves.toBe("/sprites/codex-rank-gold.png");
  });

  it("falls back to the default sheet when the art is missing", async () => {
    stubImage(() => false);
    await expect(resolveSheet("claude", "silver")).resolves.toBe(
      "/sprites/claude-fire-poison.png",
    );
  });
});
