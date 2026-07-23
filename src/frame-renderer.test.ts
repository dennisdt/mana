import { describe, expect, it, vi } from "vitest";
import type { ResolvedFrameDecoration } from "./frame-assets";
import {
  applyFrameDecoration,
  createFrameDecorationUpdater,
  frameLayerHtml,
  frameRenderPlan,
} from "./frame-renderer";

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
      ornaments: {
        top: "/rank/ornament-h.png",
        right: "/rank/ornament-v.png",
        bottom: "/rank/ornament-h.png",
        left: "/rank/ornament-v.png",
      },
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
  readonly children = new Map<string, FakeElement>();
  innerHTML = "";
  textContent = "";
  parentElement: FakeElement | null = null;

  querySelector(selector: string): FakeElement | null {
    return this.children.get(selector) ?? null;
  }
}

function fakePerimeter(): FakeElement {
  const perimeter = new FakeElement();
  const host = new FakeElement();
  perimeter.parentElement = host;
  for (const side of ["top", "right", "bottom", "left"]) {
    perimeter.children.set(
      `[data-frame-ornaments="${side}"]`,
      new FakeElement(),
    );
  }
  perimeter.children.set("[data-prestige-text]", new FakeElement());
  return perimeter;
}

describe("generated frame perimeter", () => {
  it("renders one normalized perimeter structure", () => {
    const html = frameLayerHtml();

    expect((html.match(/data-frame-rail=/g) ?? [])).toHaveLength(4);
    expect((html.match(/data-frame-corner=/g) ?? [])).toHaveLength(4);
    expect((html.match(/data-frame-ornaments=/g) ?? [])).toHaveLength(4);
    expect((html.match(/data-frame-crest/g) ?? [])).toHaveLength(1);
    expect((html.match(/data-prestige-text/g) ?? [])).toHaveLength(1);
    expect(html).not.toContain("badge");
  });

  it("uses one composited rail and corner set with deterministic ornament lanes", () => {
    const plan = frameRenderPlan(decoration());

    expect(plan.ornamentCounts).toEqual({
      top: 2,
      right: 1,
      bottom: 2,
      left: 1,
    });
    expect(plan.prestigeText).toBe("X");
    expect(plan.cssVariables).toMatchObject({
      "--frame-rank-rail-top": 'url("/rank/rail-h.png")',
      "--frame-prestige-rail-top": 'url("/prestige/rail-h.png")',
      "--frame-rank-corner-tl": 'url("/rank/corner-tl.png")',
      "--frame-prestige-corner-tl": 'url("/prestige/corner-tl.png")',
      "--frame-crest": 'url("/prestige/crest-top.png")',
      "--frame-ornament-top": 'url("/rank/ornament-h.png")',
    });
  });

  it("lets prestige ornaments replace rank ornaments in the same four lanes", () => {
    const model = decoration();
    model.prestige!.ornaments = {
      top: "/prestige/ornament-h.png",
      right: "/prestige/ornament-v.png",
      bottom: "/prestige/ornament-h.png",
      left: "/prestige/ornament-v.png",
    };

    const plan = frameRenderPlan(model);

    expect(plan.cssVariables["--frame-ornament-top"]).toBe(
      'url("/prestige/ornament-h.png")',
    );
    expect(plan.cssVariables["--frame-ornament-left"]).toBe(
      'url("/prestige/ornament-v.png")',
    );
    expect(plan.ornamentCounts).toEqual({
      top: 2,
      right: 1,
      bottom: 2,
      left: 1,
    });
  });

  it("applies one stable set of nodes without accumulating decorations", () => {
    const perimeter = fakePerimeter();
    const model = decoration();

    applyFrameDecoration(perimeter as unknown as HTMLElement, model);
    applyFrameDecoration(perimeter as unknown as HTMLElement, model);

    expect(perimeter.dataset).toMatchObject({
      rank: "godlike",
      prestige: "10",
    });
    expect(perimeter.parentElement?.dataset.frameArt).toBe("true");
    expect(perimeter.children.get('[data-frame-ornaments="top"]')?.innerHTML)
      .toBe(
        '<span class="frame-ornament" aria-hidden="true"></span>'.repeat(2),
      );
    expect(perimeter.children.get('[data-frame-ornaments="right"]')?.innerHTML)
      .toBe('<span class="frame-ornament" aria-hidden="true"></span>');
    expect(perimeter.children.get("[data-prestige-text]")?.textContent).toBe(
      "X",
    );
  });

  it("keeps only the newest asynchronous rank and prestige update", async () => {
    const perimeter = fakePerimeter();
    const releases: Array<(model: ResolvedFrameDecoration) => void> = [];
    const resolver = vi.fn(
      () =>
        new Promise<ResolvedFrameDecoration>((resolve) => {
          releases.push(resolve);
        }),
    );
    const applied: string[] = [];
    const update = createFrameDecorationUpdater(
      perimeter as unknown as HTMLElement,
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
