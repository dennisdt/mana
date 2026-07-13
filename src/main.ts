import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { fmtAge, fmtCountdown, manaLeft, planLabel } from "./format";
import { cardHtml, pillHtml, type Snapshot } from "./view";
import {
  COLLAPSED_HEIGHT,
  COLLAPSED_WIDTH,
  collapsedOriginFromExpanded,
  createHoverIntent,
  createRequestRevision,
  createSerialQueue,
  expandedHeight,
  expandedOrigin,
  EXPANDED_WIDTH,
} from "./window-layout";

type Activity = { claude: boolean; codex: boolean };

const COLLAPSED = new LogicalSize(COLLAPSED_WIDTH, COLLAPSED_HEIGHT);

const snapshots = new Map<string, Snapshot>();

const activity: Record<string, boolean> = { claude: false, codex: false };
let hovering = false;
let moving = false;
let expanded = false;
let collapsedOrigin: PhysicalPosition | undefined;
let expandedOffsetX = 0;

function spriteState(provider: string): string {
  if (moving || hovering) return "hover";
  if (activity[provider]) return "working";
  return "idle";
}

function updateSprites(): void {
  for (const provider of ["claude", "codex"]) {
    document
      .querySelectorAll<HTMLElement>(`.sprite[data-provider="${provider}"]`)
      .forEach((element) => {
        element.dataset.state = spriteState(provider);
      });
  }
}

function applyData(pill: HTMLElement, card: HTMLElement, s: Snapshot): void {
  for (const root of [pill, card]) {
    s.bars.forEach((b, i) => {
      const left = manaLeft(b.used_percent);
      root.querySelectorAll<HTMLElement>(`.track[data-bar="${i}"]`).forEach((track) => {
        track.classList.toggle("low", left < 30);
        const fill = track.querySelector<HTMLElement>(".fill");
        if (fill) fill.style.width = `${left}%`;
      });
      root.querySelectorAll<HTMLElement>(`.pct[data-bar="${i}"]`).forEach((el) => {
        el.textContent = `${Math.round(left)}%`;
      });
      root.querySelectorAll<HTMLElement>(`.cd[data-bar="${i}"]`).forEach((el) => {
        el.dataset.resets = b.resets_at == null ? "" : String(b.resets_at);
      });
    });
  }
  const plan = card.querySelector<HTMLElement>(".plan");
  if (plan) plan.textContent = planLabel(s.plan);
  const age = card.querySelector<HTMLElement>(".age");
  if (age) age.dataset.age = s.status === "stale" ? String(s.fetched_at) : "";
}

function renderProvider(provider: string): void {
  const s = snapshots.get(provider);
  const pill = document.getElementById(`pill-${provider}`)!;
  const card = document.getElementById(`card-${provider}`)!;
  const key =
    s && s.status !== "absent" && s.bars.length > 0
      ? s.bars.map((b) => `${b.id}:${b.label}`).join(",")
      : "absent";
  if (pill.dataset.key !== key) {
    pill.dataset.key = key;
    card.dataset.key = key;
    pill.innerHTML = pillHtml(s, provider);
    card.innerHTML = cardHtml(s, provider);
  }
  const stale = s?.status === "stale";
  pill.classList.toggle("stale", stale === true);
  card.classList.toggle("stale", stale === true);
  if (s && key !== "absent") applyData(pill, card, s);
  tick();
  updateSprites();
  resizeExpandedContent();
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

const enqueueSizing = createSerialQueue((error) => {
  console.error("[mana] window sizing failed", error);
});
const requestRevision = createRequestRevision();

function nextLayoutFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()));
}

function resizeExpandedContent(): void {
  if (!expanded) return;
  void enqueueSizing(async () => {
    if (!expanded) return;
    const card = document.getElementById("card")!;
    const win = getCurrentWindow();
    const position = await win.outerPosition();
    await win.setSize(
      new LogicalSize(EXPANDED_WIDTH, expandedHeight(card.scrollHeight)),
    );
    await win.setPosition(position);
  });
}

function setExpanded(on: boolean): void {
  if (expanded === on) return;
  expanded = on;
  const revision = requestRevision.issue();
  void enqueueSizing(async () => {
    if (expanded !== on) return;
    if (on && document.body.classList.contains("expanded")) return;
    const win = getCurrentWindow();
    if (on) {
      try {
        const origin = await win.outerPosition();
        const monitor = await currentMonitor();
        const target = monitor
          ? expandedOrigin(origin, {
              x: monitor.workArea.position.x,
              y: monitor.workArea.position.y,
              width: monitor.workArea.size.width,
              height: monitor.workArea.size.height,
            }, monitor.scaleFactor)
          : origin;
        collapsedOrigin = origin;
        expandedOffsetX = target.x - origin.x;
        document.body.classList.add("expanded");
        await win.setSize(new LogicalSize(EXPANDED_WIDTH, COLLAPSED_HEIGHT));
        await win.setPosition(new PhysicalPosition(target.x, target.y));
        await nextLayoutFrame();
        if (expanded !== on) return;
        const card = document.getElementById("card")!;
        await win.setSize(new LogicalSize(EXPANDED_WIDTH, expandedHeight(card.scrollHeight)));
        await win.setPosition(new PhysicalPosition(target.x, target.y));
      } catch (error) {
        if (requestRevision.isCurrent(revision) && expanded === on) expanded = false;
        document.body.classList.remove("expanded");
        await win.setSize(COLLAPSED).catch(() => undefined);
        if (collapsedOrigin) await win.setPosition(collapsedOrigin).catch(() => undefined);
        collapsedOrigin = undefined;
        expandedOffsetX = 0;
        throw error;
      }
    } else {
      let restoreOrigin = collapsedOrigin;
      if (restoreOrigin) {
        try {
          const position = await win.outerPosition();
          const restored = collapsedOriginFromExpanded(position, expandedOffsetX);
          restoreOrigin = new PhysicalPosition(restored.x, restored.y);
        } catch (error) {
          console.error("[mana] could not capture expanded position", error);
        }
      }
      document.body.classList.remove("expanded");
      try {
        await win.setSize(COLLAPSED);
        if (restoreOrigin) await win.setPosition(restoreOrigin);
      } finally {
        collapsedOrigin = undefined;
        expandedOffsetX = 0;
      }
    }
  });
}

const hoverIntent = createHoverIntent(
  (value) => {
    hovering = value;
    updateSprites();
  },
  setExpanded,
  () => moving,
);
document.body.addEventListener("mouseenter", hoverIntent.enter);
document.body.addEventListener("mouseleave", hoverIntent.leave);

let moveTimer: ReturnType<typeof setTimeout> | undefined;
void getCurrentWindow().onMoved(() => {
  moving = true;
  updateSprites();
  clearTimeout(moveTimer);
  moveTimer = setTimeout(() => {
    moving = false;
    updateSprites();
    hoverIntent.movementSettled();
  }, 300);
});

void listen<Snapshot>("usage-update", (e) => {
  snapshots.set(e.payload.provider, e.payload);
  renderProvider(e.payload.provider);
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

void invoke<Activity>("get_activity").then((a) => {
  activity.claude = a.claude ?? false;
  activity.codex = a.codex ?? false;
  updateSprites();
});

for (const provider of ["claude", "codex"]) renderProvider(provider);

setInterval(tick, 1000);
