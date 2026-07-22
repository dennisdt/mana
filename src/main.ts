import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { fmtAge, fmtCountdown, manaLeft, planLabel } from "./format";
import { meterFillPixels } from "./meter";
import { badgeSlots, levelLabel, xpBarFraction, type Progress } from "./progress-view";
import {
  spriteFrameAt,
  spriteFrameDelayAt,
  spritePhaseCycles,
} from "./sprite-animation";
import { cardHtml, providerIsVisible, type Snapshot } from "./view";
import {
  createSerialQueue,
  restoreScale,
  rosterOrigin,
  scaleForWidth,
  scaledRosterSize,
} from "./window-layout";

type Activity = { claude: boolean; codex: boolean };

const snapshots = new Map<string, Snapshot>();

const activity: Record<string, boolean> = { claude: false, codex: false };
let hovering = false;
let moving = false;
const spriteMotionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");
let spriteFrameTimer: ReturnType<typeof setTimeout> | undefined;

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
    s && s.status !== "absent" && s.bars.length > 0
      ? s.bars.map((b) => `${b.id}:${b.label}`).join(",")
      : "absent";
  if (card.dataset.key !== key) {
    card.dataset.key = key;
    card.innerHTML = cardHtml(s, provider);
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

function badgeHtml(n: number, prestige: number): string {
  // Beyond ten prestiges the tenth badge carries the total as an overlay.
  const count = n === 10 && prestige > 10 ? ` data-count="${prestige}"` : "";
  return `<span class="badge" data-n="${n}"${count} aria-hidden="true"></span>`;
}

function renderProgress(p: Progress): void {
  document.getElementById("root")!.dataset.rank = p.tier;
  const footer = document.getElementById("progress")!;
  const badges = footer.querySelector<HTMLElement>(".badges")!;
  const key = String(p.prestige);
  if (badges.dataset.key !== key) {
    badges.dataset.key = key;
    badges.innerHTML = badgeSlots(p.prestige)
      .map((n) => badgeHtml(n, p.prestige))
      .join("");
  }
  footer.querySelector<HTMLElement>(".level")!.textContent = levelLabel(p);
  footer.querySelector<HTMLElement>(".xpfill")!.style.width =
    `${xpBarFraction(p) * 100}%`;
  resizeRosterContent();
}

const SCALE_STORAGE_KEY = "mana.scale";
let scale = restoreScale(localStorage.getItem(SCALE_STORAGE_KEY));
let appliedSize: { width: number; height: number } | undefined;

function applyScale(): void {
  document.documentElement.style.zoom = String(scale);
}

applyScale();

const enqueueSizing = createSerialQueue((error) => {
  console.error("[mana] window sizing failed", error);
});

function resizeRosterContent(): void {
  void enqueueSizing(async () => {
    // Measure the wrapper, not #card, so the progress footer counts toward
    // the window height. #root itself always fills the viewport, so its
    // scrollHeight would just echo the current window size back.
    const content = document.getElementById("content")!;
    const size = scaledRosterSize(content.scrollHeight, scale);
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
    appliedSize = size;
    await win.setSize(new LogicalSize(size.width, size.height));
    await win.setPosition(new PhysicalPosition(target.x, target.y));
  });
}

document.body.addEventListener("mouseenter", () => {
  hovering = true;
  updateSprites();
});
document.body.addEventListener("mouseleave", () => {
  hovering = false;
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

let resizeTimer: ReturnType<typeof setTimeout> | undefined;
void getCurrentWindow().onResized(({ payload }) => {
  if (!appliedSize) return;
  const logical = payload.toLogical(window.devicePixelRatio);
  const programmatic =
    Math.abs(logical.width - appliedSize.width) <= 1 &&
    Math.abs(logical.height - appliedSize.height) <= 1;
  if (programmatic) return;
  scale = scaleForWidth(logical.width);
  applyScale();
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    localStorage.setItem(SCALE_STORAGE_KEY, String(scale));
    resizeRosterContent();
  }, 250);
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
setInterval(tick, 1000);
