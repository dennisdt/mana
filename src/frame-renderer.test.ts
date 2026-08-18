import { describe, expect, it, vi } from "vitest";
import type { ResolvedFrameDecoration } from "./frame-assets";
import { applyPrestigeCrest, createPrestigeCrestUpdater } from "./frame-renderer";

function decoration(
  overrides: Partial<ResolvedFrameDecoration> = {},
): ResolvedFrameDecoration {
  return {
    requestedTier: "godlike",
    resolvedTier: "godlike",
    rank: {
      key: "rank-godlike",
      rails: {
        top: "/rank/rail-h.png",
        right: "/rank/rail-v.png",
        bottom: "/rank/rail-h.png",
        left: "/rank/rail-v.png",
      },
      corners: {
        tl: "/rank/corner-tl.png",
        tr: "/rank/corner-tr.png",
        bl: "/rank/corner-bl.png",
        br: "/rank/corner-br.png",
      },
      ornaments: {},
      crestTop: "/rank/crest-top.png",
    },
    prestige: {
      key: "prestige-10",
      rails: {
        top: "/prestige/rail-h.png",
        right: "/prestige/rail-v.png",
        bottom: "/prestige/rail-h.png",
        left: "/prestige/rail-v.png",
      },
      corners: {
        tl: "/prestige/corner-tl.png",
        tr: "/prestige/corner-tr.png",
        bl: "/prestige/corner-bl.png",
        br: "/prestige/corner-br.png",
      },
      ornaments: {},
      crestTop: "/prestige/crest-top.png",
    },
    prestigeText: "X",
    diagnostics: [],
    ...overrides,
  };
}

class FakeStyle {
  readonly values = new Map<string, string>();

  setProperty(name: string, value: string): void {
    this.values.set(name, value);
  }
}

class FakeElement {
  readonly dataset: Record<string, string> = {};
  readonly style = new FakeStyle();
}

describe("prestige crest decoration", () => {
  it("marks the root and points the crest variable at prestige art", () => {
    const root = new FakeElement();

    applyPrestigeCrest(root as unknown as HTMLElement, decoration());

    expect(root.style.values.get("--progress-prestige-crest")).toBe(
      'url("/prestige/crest-top.png")',
    );
    expect(root.dataset.prestigeCrest).toBe("true");
  });

  it("hides the crest when the pilot has no prestige", () => {
    const root = new FakeElement();

    applyPrestigeCrest(
      root as unknown as HTMLElement,
      decoration({ prestige: null, prestigeText: "" }),
    );

    expect(root.style.values.get("--progress-prestige-crest")).toBe("none");
    expect(root.dataset.prestigeCrest).toBe("false");
  });

  it("keeps only the newest asynchronous rank and prestige update", async () => {
    const root = new FakeElement();
    const releases: Array<(model: ResolvedFrameDecoration) => void> = [];
    const resolver = vi.fn(
      () =>
        new Promise<ResolvedFrameDecoration>((resolve) => {
          releases.push(resolve);
        }),
    );
    const applied: string[] = [];
    const update = createPrestigeCrestUpdater(
      root as unknown as HTMLElement,
      resolver,
      (_element, model) => applied.push(model.resolvedTier),
    );

    const first = update("silver", 0);
    const second = update("godlike", 10);
    releases[1](decoration());
    await second;
    releases[0](
      decoration({
        requestedTier: "silver",
        resolvedTier: "silver",
        prestige: null,
        prestigeText: "",
      }),
    );
    await first;

    expect(applied).toEqual(["godlike"]);
  });
});
