import { resolveAura, type AuraDescriptor, type Provider } from "./aura-assets";
import { auraFrameAt, auraFrameDelayAt } from "./aura-animation";
import { rankSheetUrl } from "./cosmetics";
import {
  createFrameDecorationUpdater,
  frameLayerHtml,
} from "./frame-renderer";
import { manaLeft, planLabel } from "./format";
import { RANK_TIERS, type RankTier } from "./frame-assets";
import {
  levelLabel,
  lifetimeOutputLabel,
  type Progress,
} from "./progress-view";
import {
  spriteFrameAt,
  spriteFrameDelayAt,
  spritePhaseCycles,
} from "./sprite-animation";
import { cardHtml, type Snapshot } from "./view";
import { FRAME_BLEED, rosterHeight } from "./window-layout";

export type PreviewProviders = Provider | "both";

export type PreviewOptions = {
  rank: RankTier;
  prestige: number;
  providers: PreviewProviders;
  reducedMotion: boolean;
  outputTokens: string;
  hovering: boolean;
};

const DEFAULT_OPTIONS: PreviewOptions = Object.freeze({
  rank: "godlike",
  prestige: 10,
  providers: "both",
  reducedMotion: false,
  outputTokens: "12345678",
  hovering: false,
});

const U64_MAX = 18_446_744_073_709_551_615n;
const UNSIGNED_DECIMAL = /^(?:0|[1-9][0-9]*)$/;

const PREVIEW_LEVELS: Readonly<Record<RankTier, number>> = Object.freeze({
  naked: 1,
  plastic: 8,
  wood: 16,
  iron: 24,
  bronze: 32,
  silver: 40,
  gold: 48,
  platinum: 56,
  emerald: 64,
  diamond: 72,
  master: 80,
  legend: 88,
  champion: 94,
  godlike: 100,
});

const RESET_LABELS: Readonly<Record<Provider, readonly string[]>> = Object.freeze({
  claude: Object.freeze([" · 1h 21m", " · Tue 1:59 PM", " · Tue 1:59 PM"]),
  codex: Object.freeze([" · Wed 2:26 PM"]),
});

function parseOutputTokens(value: string | null): string {
  if (value === null || !UNSIGNED_DECIMAL.test(value)) {
    return DEFAULT_OPTIONS.outputTokens;
  }

  try {
    return BigInt(value) <= U64_MAX ? value : DEFAULT_OPTIONS.outputTokens;
  } catch {
    return DEFAULT_OPTIONS.outputTokens;
  }
}

export function parsePreviewOptions(params: URLSearchParams): PreviewOptions {
  const requestedRank = params.get("rank")?.trim().toLowerCase() ?? "";
  const rank = (RANK_TIERS as readonly string[]).includes(requestedRank)
    ? (requestedRank as RankTier)
    : DEFAULT_OPTIONS.rank;
  const prestigeParam = params.get("prestige");
  const requestedPrestige =
    prestigeParam === null ? Number.NaN : Number(prestigeParam);
  const prestige =
    Number.isSafeInteger(requestedPrestige) &&
    requestedPrestige >= 0 &&
    requestedPrestige <= 10
      ? requestedPrestige
      : DEFAULT_OPTIONS.prestige;
  const requestedProviders = params.get("providers")?.trim().toLowerCase();
  const providers =
    requestedProviders === "claude" || requestedProviders === "codex"
      ? requestedProviders
      : DEFAULT_OPTIONS.providers;

  return {
    rank,
    prestige,
    providers,
    reducedMotion: params.get("motion") === "reduced",
    outputTokens: parseOutputTokens(params.get("outputTokens")),
    hovering: params.get("hover") === "true",
  };
}

export function fixedPreviewSnapshot(provider: Provider): Snapshot {
  if (provider === "claude") {
    return {
      authenticated: true,
      provider,
      plan: "max",
      status: "ok",
      fetched_at: 0,
      bars: [
        { id: "five-hour", label: "5 hour", used_percent: 12, resets_at: null },
        { id: "weekly", label: "Weekly", used_percent: 28, resets_at: null },
        { id: "fable", label: "Fable", used_percent: 51, resets_at: null },
      ],
    };
  }

  return {
    authenticated: true,
    provider,
    plan: "pro",
    status: "ok",
    fetched_at: 0,
    bars: [
      { id: "weekly", label: "Weekly", used_percent: 6, resets_at: null },
    ],
  };
}

export function previewLevelLabel(tier: RankTier, prestige: number): string {
  return levelLabel(
    fixedPreviewProgress(tier, prestige, DEFAULT_OPTIONS.outputTokens),
  );
}

export function fixedPreviewProgress(
  tier: RankTier,
  prestige: number,
  outputTokens: string,
): Progress {
  return {
    xp: 84,
    level: PREVIEW_LEVELS[tier],
    rank: RANK_TIERS.indexOf(tier),
    tier,
    prestige,
    lifetime_output_tokens: outputTokens,
    rank_up_eligible: false,
    prestige_eligible: false,
    level_progress: { current: 84, needed: 100 },
  };
}

function populateCard(card: HTMLElement, snapshot: Snapshot): void {
  card.innerHTML = cardHtml(snapshot, snapshot.provider);
  snapshot.bars.forEach((bar, index) => {
    const percent = card.querySelector<HTMLElement>(
      `.pct[data-bar="${index}"]`,
    );
    const countdown = card.querySelector<HTMLElement>(
      `.cd[data-bar="${index}"]`,
    );
    if (percent) percent.textContent = `${Math.round(manaLeft(bar.used_percent))}%`;
    if (countdown) {
      countdown.textContent =
        RESET_LABELS[snapshot.provider as Provider][index] ?? "";
    }
  });
  const plan = card.querySelector<HTMLElement>(".plan");
  if (plan) plan.textContent = planLabel(snapshot.plan);
}

function applyAura(
  element: HTMLElement,
  descriptor: AuraDescriptor | null,
): void {
  element.hidden = descriptor === null;
  element.dataset.frame = "0";
  if (!descriptor) return;
  element.dataset.band = descriptor.band;
  element.dataset.frameCount = String(descriptor.frameCount);
  element.style.setProperty("--aura-atlas", `url("${descriptor.atlasUrl}")`);
  element.style.setProperty(
    "--aura-frame-count",
    String(descriptor.frameCount),
  );
}

function visibleProviders(options: PreviewOptions): Provider[] {
  return options.providers === "both"
    ? ["claude", "codex"]
    : [options.providers];
}

function sizePreview(root: HTMLElement): void {
  const content = document.getElementById("content");
  if (!content) return;
  const height = rosterHeight(content.offsetHeight) + FRAME_BLEED * 2;
  root.style.height = `${height}px`;
  document.documentElement.style.height = `${height}px`;
  document.body.style.height = `${height}px`;
}

function populateProgressFooter(progress: Progress): void {
  const footer = document.getElementById("progress");
  if (!footer) return;

  let crest = footer.querySelector<HTMLElement>(".prestige-crest");
  if (!crest) {
    crest = document.createElement("span");
    crest.className = "prestige-crest";
    crest.setAttribute("aria-hidden", "true");
    footer.prepend(crest);
  }

  let copy = footer.querySelector<HTMLElement>(".progress-copy");
  if (!copy) {
    const level = footer.querySelector<HTMLElement>(".level");
    copy = document.createElement("span");
    copy.className = "progress-copy";
    if (level) copy.append(level);
    footer.insertBefore(copy, footer.querySelector(".xpbar"));
  }

  let level = copy.querySelector<HTMLElement>(".level");
  if (!level) {
    level = document.createElement("span");
    level.className = "level";
    copy.append(level);
  }

  const xpbar = footer.querySelector<HTMLElement>(".xpbar");
  let lifetime = xpbar?.querySelector<HTMLElement>(".lifetime-output");
  if (!lifetime) {
    lifetime = document.createElement("span");
    lifetime.id = "lifetime-output-tooltip";
    lifetime.className = "lifetime-output";
    lifetime.setAttribute("role", "tooltip");
    xpbar?.append(lifetime);
  }

  level.textContent = levelLabel(progress);
  lifetime.textContent = lifetimeOutputLabel(progress.lifetime_output_tokens);
}

function animatePreview(
  options: PreviewOptions,
  descriptors: ReadonlyMap<Provider, AuraDescriptor | null>,
): void {
  let timer: ReturnType<typeof setTimeout> | undefined;

  const update = (now: number): void => {
    const delays: number[] = [];

    document.querySelectorAll<HTMLElement>(".sprite").forEach((element) => {
      const phase = spritePhaseCycles(element.dataset.provider);
      element.dataset.frame = String(
        spriteFrameAt(now, "idle", options.reducedMotion, phase),
      );
      const delay = spriteFrameDelayAt(
        now,
        "idle",
        options.reducedMotion,
        phase,
      );
      if (delay !== undefined) delays.push(delay);
    });

    document
      .querySelectorAll<HTMLElement>(".aura:not([hidden])")
      .forEach((element) => {
        const provider = element.dataset.provider as Provider;
        const descriptor = descriptors.get(provider);
        if (!descriptor) return;
        element.dataset.frame = String(
          auraFrameAt(now, descriptor, options.reducedMotion),
        );
        const delay = auraFrameDelayAt(
          now,
          descriptor,
          options.reducedMotion,
        );
        if (delay !== undefined) delays.push(delay);
      });

    clearTimeout(timer);
    const delay = Math.min(...delays);
    if (Number.isFinite(delay)) {
      timer = setTimeout(() => update(performance.now()), delay);
    }
  };

  update(performance.now());
}

export async function mountPreview(options: PreviewOptions): Promise<void> {
  const root = document.getElementById("root");
  const perimeter = document.getElementById("perimeter");
  if (!root || !perimeter) return;

  root.dataset.rank = options.rank;
  root.dataset.previewReduced = String(options.reducedMotion);
  perimeter.innerHTML = frameLayerHtml();

  const activeProviders = visibleProviders(options);
  const descriptors = new Map<Provider, AuraDescriptor | null>();
  for (const provider of ["claude", "codex"] as const) {
    const card = document.getElementById(`card-${provider}`);
    if (!card) continue;
    const visible = activeProviders.includes(provider);
    card.hidden = !visible;
    if (!visible) continue;

    const snapshot = fixedPreviewSnapshot(provider);
    populateCard(card, snapshot);
    const sprite = card.querySelector<HTMLElement>(".sprite");
    if (sprite) {
      sprite.style.setProperty(
        "--sprite-sheet",
        `url("${rankSheetUrl(provider, options.rank)}")`,
      );
    }
    const descriptor = resolveAura(provider, options.rank, options.prestige);
    descriptors.set(provider, descriptor);
    const aura = card.querySelector<HTMLElement>(".aura");
    if (aura) applyAura(aura, descriptor);
  }

  populateProgressFooter(
    fixedPreviewProgress(options.rank, options.prestige, options.outputTokens),
  );
  const xp = document.querySelector<HTMLElement>("#progress .xpfill");
  if (xp) xp.style.width = "84%";
  const xpbar = document.querySelector<HTMLElement>("#progress .xpbar");
  if (xpbar) xpbar.dataset.tooltipOpen = String(options.hovering);

  const updateFrame = createFrameDecorationUpdater(perimeter);
  await updateFrame(options.rank, options.prestige);
  animatePreview(options, descriptors);
  sizePreview(root);
  document.body.dataset.previewReady = "true";
}

if (
  typeof document !== "undefined" &&
  document.getElementById("preview-root")
) {
  void mountPreview(parsePreviewOptions(new URLSearchParams(location.search)));
}
