import {
  resolveFrameDecoration,
  type ResolvedFrameDecoration,
} from "./frame-assets";

type CrestResolver = (
  tier: string,
  prestige: number,
) => Promise<ResolvedFrameDecoration>;

type CrestApplier = (
  root: HTMLElement,
  model: ResolvedFrameDecoration,
) => void;

function cssUrl(path: string | undefined): string {
  if (!path) return "none";
  const escaped = path.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `url("${escaped}")`;
}

/* The perimeter frame art retired when the window became a native Liquid
   Glass sheet; only the progress footer's prestige crest survives from the
   generated-art pipeline. */
export function applyPrestigeCrest(
  root: HTMLElement,
  model: ResolvedFrameDecoration,
): void {
  root.style.setProperty(
    "--progress-prestige-crest",
    cssUrl(model.prestige?.crestTop),
  );
  root.dataset.prestigeCrest = String(model.prestige !== null);
}

export function createPrestigeCrestUpdater(
  root: HTMLElement,
  resolve: CrestResolver = resolveFrameDecoration,
  apply: CrestApplier = applyPrestigeCrest,
): (tier: string, prestige: number) => Promise<void> {
  let revision = 0;

  return async (tier: string, prestige: number): Promise<void> => {
    const requestedRevision = ++revision;
    const model = await resolve(tier, prestige);
    if (requestedRevision !== revision) return;

    apply(root, model);
    for (const diagnostic of model.diagnostics) {
      console.warn(`[mana] ${diagnostic}`);
    }
  };
}
