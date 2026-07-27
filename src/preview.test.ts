import { describe, expect, it } from "vitest";
// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync } from "node:fs";
import previewMarkup from "../preview.html?raw";
import {
  fixedPreviewSnapshot,
  parsePreviewOptions,
  previewLevelLabel,
} from "./preview";

describe("browser-only visual preview", () => {
  it("defaults to the final Godlike and Prestige X state", () => {
    expect(parsePreviewOptions(new URLSearchParams())).toEqual({
      rank: "godlike",
      prestige: 10,
      providers: "both",
      reducedMotion: false,
      outputTokens: "12345678",
      hovering: false,
    });
  });

  it("accepts every authored rank and supported provider filter", () => {
    expect(
      parsePreviewOptions(
        new URLSearchParams(
          "rank=bronze&prestige=0&providers=claude&motion=reduced",
        ),
      ),
    ).toEqual({
      rank: "bronze",
      prestige: 0,
      providers: "claude",
      reducedMotion: true,
      outputTokens: "12345678",
      hovering: false,
    });
    expect(
      parsePreviewOptions(
        new URLSearchParams("rank=emerald&prestige=7&providers=codex"),
      ),
    ).toMatchObject({
      rank: "emerald",
      prestige: 7,
      providers: "codex",
    });
  });

  it("normalizes invalid inputs without exposing ungenerated art paths", () => {
    expect(
      parsePreviewOptions(
        new URLSearchParams(
          "rank=mythic&prestige=999&providers=none&motion=spin",
        ),
      ),
    ).toEqual({
      rank: "godlike",
      prestige: 10,
      providers: "both",
      reducedMotion: false,
      outputTokens: "12345678",
      hovering: false,
    });
    expect(
      parsePreviewOptions(new URLSearchParams("prestige=-1.5")),
    ).toMatchObject({ prestige: 10 });
  });

  it("uses fixed readable usage rows with no credential or clock dependency", () => {
    const claude = fixedPreviewSnapshot("claude");
    const codex = fixedPreviewSnapshot("codex");

    expect(claude.authenticated).toBe(true);
    expect(claude.bars.map((bar) => bar.label)).toEqual([
      "5 hour",
      "Weekly",
      "Fable",
    ]);
    expect(codex.bars.map((bar) => bar.label)).toEqual(["Weekly"]);
    expect(claude.bars.every((bar) => bar.resets_at === null)).toBe(true);
    expect(codex.bars.every((bar) => bar.resets_at === null)).toBe(true);
  });

  it("labels prestige explicitly while preserving the rank name", () => {
    expect(previewLevelLabel("godlike", 10)).toBe(
      "Lv 100 · Godlike · Prestige X",
    );
    expect(previewLevelLabel("bronze", 0)).toBe("Lv 32 · Bronze");
  });

  it("mounts a separate Vite entry without importing the native runtime", () => {
    const source = readFileSync(new URL("./preview.ts", import.meta.url), "utf8");

    expect(previewMarkup).toContain('id="preview-root"');
    expect(previewMarkup).toContain('src="/src/preview.ts"');
    expect(previewMarkup).not.toContain('src="/src/main.ts"');
    expect(source).not.toContain("@tauri-apps");
    expect(source).not.toMatch(/\binvoke\s*\(/);
    expect(source).not.toMatch(/\blisten\s*\(/);
    expect(source).not.toContain("localStorage");
    expect(source).not.toContain("credentials");
    expect(source).not.toContain("get_progress");
    expect(source).not.toContain("get_snapshots");
  });

  it("disables fill transitions before reduced-motion preview readiness", () => {
    expect(previewMarkup).toMatch(
      /#root\[data-preview-reduced="true"\] \.fill,\s*#root\[data-preview-reduced="true"\] #progress \.xpfill\s*\{[^}]*transition:\s*none\s*!important;/s,
    );
  });
});
