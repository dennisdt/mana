import {
  resolveFrameDecoration,
  type FrameCorner,
  type FrameSide,
  type ResolvedFrameDecoration,
} from "./frame-assets";

export { FRAME_BLEED } from "./window-layout";

const SIDES: readonly FrameSide[] = ["top", "right", "bottom", "left"];
const CORNERS: readonly FrameCorner[] = ["tl", "tr", "bl", "br"];
const ORNAMENT_COUNTS: Record<FrameSide, number> = {
  top: 2,
  right: 1,
  bottom: 2,
  left: 1,
};

type FrameRenderPlan = {
  cssVariables: Record<string, string>;
  hasCornerEmblem: boolean;
  hasCornerSurface: boolean;
  ornamentCounts: Record<FrameSide, number>;
  /** @deprecated Compatibility metadata; it is never rendered. */
  prestigeText: string;
};

type FrameResolver = (
  tier: string,
  prestige: number,
) => Promise<ResolvedFrameDecoration>;

type FrameApplier = (
  perimeter: HTMLElement,
  model: ResolvedFrameDecoration,
) => void;

function cssUrl(path: string | undefined): string {
  if (!path) return "none";
  const escaped = path.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `url("${escaped}")`;
}

function prestigeLevel(model: ResolvedFrameDecoration): string {
  const match = /^prestige-(\d+)$/.exec(model.prestige?.key ?? "");
  return match?.[1] ?? "0";
}

export function frameLayerHtml(): string {
  const rails = SIDES.map(
    (side) =>
      `<span class="frame-rail frame-rail--${side}" data-frame-rail="${side}"></span>`,
  ).join("");
  const corners = CORNERS.map(
    (corner) =>
      `<span class="frame-corner frame-corner--${corner}" data-frame-corner="${corner}"></span>`,
  ).join("");
  const ornaments = SIDES.map(
    (side) =>
      `<span class="frame-ornaments frame-ornaments--${side}" data-frame-ornaments="${side}"></span>`,
  ).join("");

  return `${rails}${corners}${ornaments}<span class="frame-crest" data-frame-crest></span>`;
}

export function frameRenderPlan(
  model: ResolvedFrameDecoration,
): FrameRenderPlan {
  const cssVariables: Record<string, string> = {};
  const ornamentCounts = {} as Record<FrameSide, number>;
  const cornerEmblem = model.prestige ? undefined : model.rank?.crestTop;
  const cornerSurfaces = model.prestige?.cornerSurfaces;
  const hasCornerSurface = CORNERS.every((corner) =>
    Boolean(cornerSurfaces?.[corner]),
  );

  for (const side of SIDES) {
    cssVariables[`--frame-rank-rail-${side}`] = cssUrl(
      model.rank?.rails[side],
    );
    cssVariables[`--frame-prestige-rail-${side}`] = cssUrl(
      model.prestige?.rails[side],
    );
    const ornament = model.prestige
      ? model.prestige.ornaments[side]
      : model.rank?.ornaments[side];
    cssVariables[`--frame-ornament-${side}`] = cssUrl(ornament);
    ornamentCounts[side] = ornament ? ORNAMENT_COUNTS[side] : 0;
  }

  for (const corner of CORNERS) {
    cssVariables[`--frame-rank-corner-${corner}`] = cssUrl(
      model.rank?.corners[corner],
    );
    cssVariables[`--frame-prestige-corner-${corner}`] = cssUrl(
      model.prestige?.corners[corner],
    );
    cssVariables[`--frame-corner-surface-${corner}`] = cssUrl(
      cornerSurfaces?.[corner],
    );
  }

  cssVariables["--frame-crest"] = cssUrl(model.rank?.crestTop);
  cssVariables["--frame-corner-emblem"] = cssUrl(cornerEmblem);
  cssVariables["--progress-prestige-crest"] = cssUrl(
    model.prestige?.crestTop,
  );

  return {
    cssVariables,
    hasCornerEmblem: Boolean(cornerEmblem || hasCornerSurface),
    hasCornerSurface,
    ornamentCounts,
    prestigeText: model.prestigeText,
  };
}

export function applyFrameDecoration(
  perimeter: HTMLElement,
  model: ResolvedFrameDecoration,
): void {
  const plan = frameRenderPlan(model);

  for (const [name, value] of Object.entries(plan.cssVariables)) {
    perimeter.style.setProperty(name, value);
  }

  perimeter.dataset.rank = model.resolvedTier;
  perimeter.dataset.prestige = prestigeLevel(model);
  perimeter.dataset.cornerEmblem = String(plan.hasCornerEmblem);
  perimeter.dataset.cornerSurface = String(plan.hasCornerSurface);
  if (perimeter.parentElement) {
    perimeter.parentElement.style.setProperty(
      "--progress-prestige-crest",
      plan.cssVariables["--progress-prestige-crest"],
    );
    perimeter.parentElement.dataset.frameArt = String(
      model.rank !== null || model.prestige !== null,
    );
    perimeter.parentElement.dataset.prestigeCrest = String(
      model.prestige !== null,
    );
  }

  for (const side of SIDES) {
    const lane = perimeter.querySelector<HTMLElement>(
      `[data-frame-ornaments="${side}"]`,
    );
    if (!lane) continue;
    lane.innerHTML =
      '<span class="frame-ornament" aria-hidden="true"></span>'.repeat(
        plan.ornamentCounts[side],
      );
  }
}

export function createFrameDecorationUpdater(
  perimeter: HTMLElement,
  resolve: FrameResolver = resolveFrameDecoration,
  apply: FrameApplier = applyFrameDecoration,
): (tier: string, prestige: number) => Promise<void> {
  let revision = 0;

  return async (tier: string, prestige: number): Promise<void> => {
    const requestedRevision = ++revision;
    const model = await resolve(tier, prestige);
    if (requestedRevision !== revision) return;

    apply(perimeter, model);
    for (const diagnostic of model.diagnostics) {
      console.warn(`[mana] ${diagnostic}`);
    }
  };
}
