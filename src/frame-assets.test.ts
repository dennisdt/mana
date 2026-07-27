import { describe, expect, it, vi } from "vitest";
import {
  RANK_TIERS,
  prestigeLabel,
  resolveFrameDecoration,
} from "./frame-assets";

describe("frame asset registry", () => {
  it("builds the exact convention-based manifest for a complete elite rank", async () => {
    const probe = vi.fn(async (_url: string) => true);

    const model = await resolveFrameDecoration("emerald", 0, probe);

    expect(model).toMatchObject({
      requestedTier: "emerald",
      resolvedTier: "emerald",
      diagnostics: [],
      prestige: null,
      prestigeText: "",
      rank: {
        key: "rank-emerald",
        rails: {
          top: "/frames/ranks/emerald/rail-h.png",
          right: "/frames/ranks/emerald/rail-v.png",
          bottom: "/frames/ranks/emerald/rail-h.png",
          left: "/frames/ranks/emerald/rail-v.png",
        },
        corners: {
          tl: "/frames/ranks/emerald/corner-tl.png",
          tr: "/frames/ranks/emerald/corner-tr.png",
          bl: "/frames/ranks/emerald/corner-bl.png",
          br: "/frames/ranks/emerald/corner-br.png",
        },
        ornaments: {
          top: "/frames/ranks/emerald/ornament-h.png",
          right: "/frames/ranks/emerald/ornament-v.png",
          bottom: "/frames/ranks/emerald/ornament-h.png",
          left: "/frames/ranks/emerald/ornament-v.png",
        },
        crestTop: "/frames/ranks/emerald/crest-top.png",
      },
    });
    expect(probe.mock.calls.map(([url]) => url)).toEqual([
      "/frames/ranks/emerald/rail-h.png",
      "/frames/ranks/emerald/rail-v.png",
      "/frames/ranks/emerald/corner-tl.png",
      "/frames/ranks/emerald/corner-tr.png",
      "/frames/ranks/emerald/corner-bl.png",
      "/frames/ranks/emerald/corner-br.png",
      "/frames/ranks/emerald/crest-top.png",
      "/frames/ranks/emerald/ornament-h.png",
      "/frames/ranks/emerald/ornament-v.png",
    ]);
  });

  it("does not require optional art for a base rank", async () => {
    const probe = vi.fn(async (_url: string) => true);

    const model = await resolveFrameDecoration("silver", 0, probe);

    expect(model.rank?.ornaments).toEqual({});
    expect(model.rank?.crestTop).toBeUndefined();
    expect(probe).toHaveBeenCalledTimes(6);
  });

  it("uses one prestige crest and clamps prestige art at ten", async () => {
    const probe = vi.fn(async (_url: string) => true);
    const model = await resolveFrameDecoration("godlike", 12, probe);

    expect(model.prestige?.crestTop).toBe("/frames/prestige/10/crest-top.png");
    expect(model.prestigeText).toBe("P12");
    expect(model.rank?.crestTop).toBe("/frames/ranks/godlike/crest-top.png");
    expect(
      probe.mock.calls
        .map(([url]) => url)
        .filter((url) => url.includes("/frames/prestige/")),
    ).toEqual([
      "/frames/prestige/10/rail-h.png",
      "/frames/prestige/10/rail-v.png",
      "/frames/prestige/10/corner-tl.png",
      "/frames/prestige/10/corner-tr.png",
      "/frames/prestige/10/corner-bl.png",
      "/frames/prestige/10/corner-br.png",
      "/frames/prestige/10/crest-top.png",
      "/frames/prestige/10/corner-joint-tl.png",
      "/frames/prestige/10/corner-joint-tr.png",
      "/frames/prestige/10/corner-joint-bl.png",
      "/frames/prestige/10/corner-joint-br.png",
    ]);
  });

  it("falls back to the nearest complete lower rank with one diagnostic per rejection", async () => {
    const model = await resolveFrameDecoration("emerald", 0, async (url) =>
      !url.includes("/emerald/") && !url.includes("/platinum/"),
    );

    expect(model.resolvedTier).toBe("gold");
    expect(model.diagnostics).toHaveLength(2);
    expect(model.diagnostics[0]).toContain("emerald");
    expect(model.diagnostics[1]).toContain("platinum");
  });

  it("continues to a viable lower fallback after false and rejected probes", async () => {
    const probe = vi.fn(async (url: string) => {
      if (url.includes("/diamond/")) return false;
      if (url.includes("/emerald/")) throw new Error("decode failed");
      return true;
    });

    const model = await resolveFrameDecoration("diamond", 0, probe);

    expect(model.resolvedTier).toBe("platinum");
    expect(model.rank?.key).toBe("rank-platinum");
    expect(model.diagnostics).toHaveLength(2);
  });

  it("falls prestige art back independently and keeps the sole top crest", async () => {
    const model = await resolveFrameDecoration("godlike", 3, async (url) => {
      if (url.includes("/prestige/3/")) return false;
      if (url.includes("/prestige/2/")) throw new Error("missing");
      return true;
    });

    expect(model.prestige?.key).toBe("prestige-1");
    expect(model.prestige?.crestTop).toBe("/frames/prestige/1/crest-top.png");
    expect(model.rank?.crestTop).toBe("/frames/ranks/godlike/crest-top.png");
    expect(model.prestigeText).toBe("III");
    expect(model.diagnostics.filter((message) => message.includes("Prestige"))).toHaveLength(2);
  });

  it("keeps the rank crest and hides prestige text when every prestige probe is false", async () => {
    const model = await resolveFrameDecoration("godlike", 3, async (url) =>
      !url.includes("/frames/prestige/"),
    );

    expect(model.rank?.crestTop).toBe("/frames/ranks/godlike/crest-top.png");
    expect(model.prestige).toBeNull();
    expect(model.prestigeText).toBe("");
    expect(model.diagnostics.filter((message) => message.includes("Prestige")))
      .toHaveLength(3);
  });

  it("keeps the rank crest and hides prestige text when every prestige probe rejects", async () => {
    const model = await resolveFrameDecoration("emerald", 2, async (url) => {
      if (url.includes("/frames/prestige/")) throw new Error("decode failed");
      return true;
    });

    expect(model.rank?.crestTop).toBe("/frames/ranks/emerald/crest-top.png");
    expect(model.prestige).toBeNull();
    expect(model.prestigeText).toBe("");
    expect(model.diagnostics.filter((message) => message.includes("Prestige")))
      .toHaveLength(2);
  });

  it("renders roman prestige labels from one through ten and counts above ten", () => {
    expect(Array.from({ length: 10 }, (_, i) => prestigeLabel(i + 1))).toEqual([
      "I",
      "II",
      "III",
      "IV",
      "V",
      "VI",
      "VII",
      "VIII",
      "IX",
      "X",
    ]);
    expect(prestigeLabel(11)).toBe("P11");
    expect(prestigeLabel(42)).toBe("P42");
  });

  it.each([
    [Number.NaN],
    [Number.POSITIVE_INFINITY],
    [-1],
    [0],
    [1.9],
  ])("normalizes invalid prestige %s to no prestige", async (prestige) => {
    const model = await resolveFrameDecoration("gold", prestige, async () => true);

    expect(model.prestige).toBeNull();
    expect(model.prestigeText).toBe("");
    expect(model.rank?.crestTop).toBe("/frames/ranks/gold/crest-top.png");
  });

  it("normalizes harmless tier casing and whitespace", async () => {
    const model = await resolveFrameDecoration(" GODLIKE ", 0, async () => true);

    expect(model.requestedTier).toBe("godlike");
    expect(model.resolvedTier).toBe("godlike");
  });

  it.each(["", "unknown", "__proto__"])(
    "normalizes an unknown tier %j to naked",
    async (tier) => {
      const model = await resolveFrameDecoration(tier, 0, async () => true);

      expect(model.requestedTier).toBe("naked");
      expect(model.resolvedTier).toBe("naked");
      expect(model.rank).toBeNull();
    },
  );

  it("keeps the public rank order stable", () => {
    expect(RANK_TIERS).toEqual([
      "naked",
      "plastic",
      "wood",
      "iron",
      "bronze",
      "silver",
      "gold",
      "platinum",
      "emerald",
      "diamond",
      "master",
      "legend",
      "champion",
      "godlike",
    ]);
  });
});
