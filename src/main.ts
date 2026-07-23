import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { resolveSheet } from "./cosmetics";
import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { fmtAge, fmtCountdown, manaLeft, planLabel } from "./format";
import { meterFillPixels } from "./meter";
import {
  actionKind,
  dialogCopy,
  levelLabel,
  lifetimeOutputLabel,
  nextTier,
  xpBarFraction,
  type Progress,
} from "./progress-view";
import {
  spriteFrameAt,
  spriteFrameDelayAt,
  spritePhaseCycles,
} from "./sprite-animation";
import { resolveAura } from "./aura-assets";
import { auraFrameAt, auraFrameDelayAt } from "./aura-animation";
import { cardHtml, providerIsVisible, type Snapshot } from "./view";
import {
  createFrameDecorationUpdater,
  frameLayerHtml,
} from "./frame-renderer";
import {
  createSerialQueue,
  rosterOrigin,
  scaledRosterSize,
  WIDGET_ZOOM,
} from "./window-layout";

type Activity = { claude: boolean; codex: boolean };

const snapshots = new Map<string, Snapshot>();

const activity: Record<string, boolean> = { claude: false, codex: false };
let hovering = false;
let moving = false;
const spriteMotionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");
let spriteFrameTimer: ReturnType<typeof setTimeout> | undefined;
const auraMotionPreference = spriteMotionPreference;
let auraFrameTimer: ReturnType<typeof setTimeout> | undefined;
const perimeter = document.getElementById("perimeter")!;
perimeter.innerHTML = frameLayerHtml();
const updateFrameDecoration = createFrameDecorationUpdater(perimeter);

function spriteState(provider: string): string {
  if (moving || hovering) return "hover";
  if (activity[provider]) return "working";
  return "idle";
}

function updateSpriteFrames(now: number = performance.now()): void {
  document.querySelectorAll<HTMLElement>(".sprite").forEach((element) => {
    const phaseCycles = spritePhaseCycles(element.dataset.provider);
    const frame = String(
      spriteFrameAt(
        now,
        element.dataset.state,
        spriteMotionPreference.matches,
        phaseCycles,
      ),
    );
    if (element.dataset.frame !== frame) element.dataset.frame = frame;
  });
}

function scheduleSpriteFrameUpdate(now: number = performance.now()): void {
  clearTimeout(spriteFrameTimer);
  const delays = Array.from(document.querySelectorAll<HTMLElement>(".sprite"), (element) => {
    const phaseCycles = spritePhaseCycles(element.dataset.provider);
    return spriteFrameDelayAt(
      now,
      element.dataset.state,
      spriteMotionPreference.matches,
      phaseCycles,
    );
  }).filter((delay): delay is number => delay !== undefined);
  const delay = Math.min(...delays);
  if (!Number.isFinite(delay)) return;
  spriteFrameTimer = setTimeout(runSpriteFrameUpdate, delay);
}

function runSpriteFrameUpdate(): void {
  const now = performance.now();
  updateSpriteFrames(now);
  scheduleSpriteFrameUpdate(now);
}

function syncSpriteFrames(now: number = performance.now()): void {
  updateSpriteFrames(now);
  scheduleSpriteFrameUpdate(now);
}

function listenForSpriteMotionPreference(): void {
  const update = () => syncSpriteFrames();
  if (typeof spriteMotionPreference.addEventListener === "function") {
    spriteMotionPreference.addEventListener("change", update);
  } else {
    spriteMotionPreference.addListener(update);
  }
}

function auraDescriptorFor(element: HTMLElement) {
  const provider = element.dataset.provider;
  if (provider !== "claude" && provider !== "codex") return null;
  return resolveAura(
    provider,
    progress?.tier ?? "naked",
    progress?.prestige ?? 0,
  );
}

function applyAuras(): void {
  document.querySelectorAll<HTMLElement>(".aura").forEach((element) => {
    const descriptor = auraDescriptorFor(element);
    element.hidden = descriptor === null;
    element.dataset.frame = "0";
    if (!descriptor) {
      delete element.dataset.band;
      delete element.dataset.frameCount;
      element.style.removeProperty("--aura-atlas");
      element.style.removeProperty("--aura-frame-count");
      return;
    }
    element.dataset.band = descriptor.band;
    element.dataset.frameCount = String(descriptor.frameCount);
    element.style.setProperty("--aura-atlas", `url("${descriptor.atlasUrl}")`);
    element.style.setProperty(
      "--aura-frame-count",
      String(descriptor.frameCount),
    );
  });
}

function updateAuraFrames(now: number = performance.now()): void {
  document.querySelectorAll<HTMLElement>(".aura:not([hidden])").forEach((element) => {
    const descriptor = auraDescriptorFor(element);
    if (!descriptor) return;
    const frame = String(
      auraFrameAt(now, descriptor, auraMotionPreference.matches),
    );
    if (element.dataset.frame !== frame) element.dataset.frame = frame;
  });
}

function scheduleAuraFrameUpdate(now: number = performance.now()): void {
  clearTimeout(auraFrameTimer);
  if (auraMotionPreference.matches) return;
  const delays = Array.from(
    document.querySelectorAll<HTMLElement>(".aura:not([hidden])"),
    (element) => {
      const descriptor = auraDescriptorFor(element);
      return descriptor
        ? auraFrameDelayAt(now, descriptor, auraMotionPreference.matches)
        : undefined;
    },
  ).filter((delay): delay is number => delay !== undefined);
  const delay = Math.min(...delays);
  if (!Number.isFinite(delay)) return;
  auraFrameTimer = setTimeout(runAuraFrameUpdate, delay);
}

function runAuraFrameUpdate(): void {
  const now = performance.now();
  updateAuraFrames(now);
  scheduleAuraFrameUpdate(now);
}

function syncAuraFrames(now: number = performance.now()): void {
  updateAuraFrames(now);
  scheduleAuraFrameUpdate(now);
}

function listenForAuraMotionPreference(): void {
  const update = () => syncAuraFrames();
  if (typeof auraMotionPreference.addEventListener === "function") {
    auraMotionPreference.addEventListener("change", update);
  } else {
    auraMotionPreference.addListener(update);
  }
}

function updateSprites(): void {
  for (const provider of ["claude", "codex"]) {
    document
      .getElementById(`card-${provider}`)
      ?.toggleAttribute("data-working", activity[provider] === true);
    document
      .querySelectorAll<HTMLElement>(`.sprite[data-provider="${provider}"]`)
      .forEach((element) => {
        element.dataset.state = spriteState(provider);
      });
  }
  syncSpriteFrames();
}

function applyData(card: HTMLElement, s: Snapshot): void {
  s.bars.forEach((b, i) => {
    const left = manaLeft(b.used_percent);
    card.querySelectorAll<HTMLElement>(`.track[data-bar="${i}"]`).forEach((track) => {
      track.classList.toggle("low", left < 30);
      const fill = track.querySelector<HTMLElement>(".fill");
      if (fill) {
        const pixels = meterFillPixels(left);
        fill.style.width = `${pixels}px`;
        fill.dataset.empty = String(pixels === 0);
      }
    });
    card.querySelectorAll<HTMLElement>(`.pct[data-bar="${i}"]`).forEach((el) => {
      el.textContent = `${Math.round(left)}%`;
    });
    card.querySelectorAll<HTMLElement>(`.cd[data-bar="${i}"]`).forEach((el) => {
      el.dataset.resets = b.resets_at == null ? "" : String(b.resets_at);
    });
  });
  const plan = card.querySelector<HTMLElement>(".plan");
  if (plan) plan.textContent = planLabel(s.plan);
  const age = card.querySelector<HTMLElement>(".age");
  if (age) age.dataset.age = s.status === "stale" ? String(s.fetched_at) : "";
}

function renderProvider(provider: string): void {
  const s = snapshots.get(provider);
  const card = document.getElementById(`card-${provider}`)!;
  card.hidden = !providerIsVisible(s);
  const key =
    !providerIsVisible(s)
      ? "unauthenticated"
      : s && s.status !== "absent" && s.bars.length > 0
      ? s.bars.map((b) => `${b.id}:${b.label}`).join(",")
      : "absent";
  if (card.dataset.key !== key) {
    card.dataset.key = key;
    card.innerHTML = cardHtml(s, provider);
    // Rebuilt sprites fall back to the CSS default sheet; re-dress them.
    applySheets();
    applyAuras();
    syncAuraFrames();
  }
  const stale = s?.status === "stale";
  card.classList.toggle("stale", stale === true);
  if (s && key !== "absent") applyData(card, s);
  tick();
  updateSprites();
  resizeRosterContent();
}

function tick(): void {
  const now = Date.now();
  document.querySelectorAll<HTMLElement>(".cd").forEach((el) => {
    const t = Number(el.dataset.resets);
    el.textContent = el.dataset.resets && t > 0 ? ` · ${fmtCountdown(t, now)}` : "";
  });
  document.querySelectorAll<HTMLElement>(".age").forEach((el) => {
    el.textContent = el.dataset.age ? fmtAge(Number(el.dataset.age), now) : "";
  });
}

const sheetUrls: Record<string, string | undefined> = {};
let sheetTier: string | undefined;

function applySheets(): void {
  for (const provider of ["claude", "codex"]) {
    const url = sheetUrls[provider];
    if (!url) continue;
    document
      .querySelectorAll<HTMLElement>(`.sprite[data-provider="${provider}"]`)
      .forEach((element) => {
        element.style.setProperty("--sprite-sheet", `url("${url}")`);
      });
  }
}

function updateRankSheets(tier: string): void {
  if (sheetTier === tier) return;
  sheetTier = tier;
  for (const provider of ["claude", "codex"]) {
    void resolveSheet(provider, tier).then((url) => {
      sheetUrls[provider] = url;
      applySheets();
    });
  }
}

let progress: Progress | undefined;

function renderProgress(p: Progress): void {
  progress = p;
  document.getElementById("root")!.dataset.rank = p.tier;
  const kind = actionKind(p);
  const action = document.getElementById("action")!;
  action.hidden = kind === null;
  if (kind) action.textContent = dialogCopy(kind, p).confirm;
  const footer = document.getElementById("progress")!;
  footer.querySelector<HTMLElement>(".level")!.textContent = levelLabel(p);
  footer.querySelector<HTMLElement>(".lifetime-output")!.textContent =
    lifetimeOutputLabel(p.lifetime_output_tokens);
  footer.querySelector<HTMLElement>(".xpfill")!.style.width =
    `${xpBarFraction(p) * 100}%`;
  updateRankSheets(p.tier);
  void updateFrameDecoration(p.tier, p.prestige);
  applyAuras();
  syncAuraFrames();
  resizeRosterContent();
}

// The 2x sprite atlases must land at one source pixel per Retina pixel.
// Counter-scale only the familiar layer if the widget zoom changes again.
document.documentElement.style.setProperty(
  "--sprite-resolution-scale",
  String(1 / WIDGET_ZOOM),
);
document.documentElement.style.zoom = String(WIDGET_ZOOM);

const enqueueSizing = createSerialQueue((error) => {
  console.error("[mana] window sizing failed", error);
});

function resizeRosterContent(): void {
  void enqueueSizing(async () => {
    // Measure the wrapper, not #card, so the progress footer counts toward
    // the window height. #root itself always fills the viewport, so its
    // scrollHeight would just echo the current window size back.
    const content = document.getElementById("content")!;
    const size = scaledRosterSize(content.scrollHeight, WIDGET_ZOOM);
    const win = getCurrentWindow();
    const position = await win.outerPosition();
    const monitor = await currentMonitor();
    const target = monitor
      ? rosterOrigin(
          position,
          size,
          {
            x: monitor.workArea.position.x,
            y: monitor.workArea.position.y,
            width: monitor.workArea.size.width,
            height: monitor.workArea.size.height,
          },
          monitor.scaleFactor,
        )
      : position;
    await win.setSize(new LogicalSize(size.width, size.height));
    await win.setPosition(new PhysicalPosition(target.x, target.y));
  });
}

document.body.addEventListener("mouseenter", () => {
  hovering = true;
  document.getElementById("root")!.dataset.hovering = "true";
  updateSprites();
});
document.body.addEventListener("mouseleave", () => {
  hovering = false;
  document.getElementById("root")!.dataset.hovering = "false";
  updateSprites();
});

let moveTimer: ReturnType<typeof setTimeout> | undefined;
void getCurrentWindow().onMoved(() => {
  moving = true;
  updateSprites();
  clearTimeout(moveTimer);
  moveTimer = setTimeout(() => {
    moving = false;
    updateSprites();
  }, 300);
});

const actionButton = document.getElementById("action")!;
const ceremony = document.getElementById("ceremony")!;

// #root is one deep Tauri drag region; interactive controls must swallow
// mousedown or every click starts a window drag. The ceremony backdrop
// deliberately stays draggable.
actionButton.addEventListener("mousedown", (event) => event.stopPropagation());
ceremony
  .querySelector<HTMLElement>(".ceremony-panel")!
  .addEventListener("mousedown", (event) => event.stopPropagation());

function openCeremony(kind: "rank-up" | "prestige", p: Progress): void {
  const copy = dialogCopy(kind, p);
  ceremony.dataset.kind = kind;
  ceremony.dataset.tier =
    kind === "rank-up" ? (nextTier(p) ?? p.tier) : `prestige-${p.prestige + 1}`;
  ceremony.querySelector<HTMLElement>("h1")!.textContent = copy.title;
  ceremony.querySelector<HTMLElement>("p")!.textContent = copy.body;
  ceremony.querySelector<HTMLElement>(".confirm")!.textContent = copy.confirm;
  ceremony.hidden = false;
}

actionButton.addEventListener("click", () => {
  if (!progress) return;
  const kind = actionKind(progress);
  if (kind) openCeremony(kind, progress);
});

ceremony.querySelector<HTMLElement>(".confirm")!.addEventListener("click", () => {
  // One ceremony per click: re-render leaves the button glowing when more
  // gates are already crossed, and the user opens the next ceremony.
  const command = ceremony.dataset.kind === "prestige" ? "prestige" : "rank_up";
  ceremony.hidden = true;
  invoke<Progress>(command)
    .then(renderProgress)
    .catch((error) => console.error(`[mana] ${command} failed`, error));
});

ceremony.querySelector<HTMLElement>(".later")!.addEventListener("click", () => {
  ceremony.hidden = true;
});

void listen<Snapshot>("usage-update", (e) => {
  snapshots.set(e.payload.provider, e.payload);
  renderProvider(e.payload.provider);
});

void listen<Progress>("progress-update", (e) => {
  renderProgress(e.payload);
});

void listen<Activity>("activity", (e) => {
  activity.claude = e.payload.claude ?? false;
  activity.codex = e.payload.codex ?? false;
  updateSprites();
});

void invoke<Snapshot[]>("get_snapshots").then((all) => {
  for (const s of all) snapshots.set(s.provider, s);
  for (const provider of ["claude", "codex"]) renderProvider(provider);
});

void invoke<Progress>("get_progress").then(renderProgress);

void invoke<Activity>("get_activity").then((a) => {
  activity.claude = a.claude ?? false;
  activity.codex = a.codex ?? false;
  updateSprites();
});

for (const provider of ["claude", "codex"]) renderProvider(provider);

listenForSpriteMotionPreference();
syncSpriteFrames();
listenForAuraMotionPreference();
syncAuraFrames();
setInterval(tick, 1000);
