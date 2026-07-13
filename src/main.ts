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
import { cardHtml, type Snapshot } from "./view";
import {
  ROSTER_WIDTH,
  createSerialQueue,
  rosterHeight,
  rosterOrigin,
} from "./window-layout";

type Activity = { claude: boolean; codex: boolean };

const snapshots = new Map<string, Snapshot>();

const activity: Record<string, boolean> = { claude: false, codex: false };
let hovering = false;
let moving = false;

function spriteState(provider: string): string {
  if (moving || hovering) return "hover";
  if (activity[provider]) return "working";
  return "idle";
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
}

function applyData(card: HTMLElement, s: Snapshot): void {
  s.bars.forEach((b, i) => {
    const left = manaLeft(b.used_percent);
    card.querySelectorAll<HTMLElement>(`.track[data-bar="${i}"]`).forEach((track) => {
      track.classList.toggle("low", left < 30);
      const fill = track.querySelector<HTMLElement>(".fill");
      if (fill) fill.style.width = `${meterFillPixels(left)}px`;
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

const enqueueSizing = createSerialQueue((error) => {
  console.error("[mana] window sizing failed", error);
});

function resizeRosterContent(): void {
  void enqueueSizing(async () => {
    const card = document.getElementById("card")!;
    const height = rosterHeight(card.scrollHeight);
    const win = getCurrentWindow();
    const position = await win.outerPosition();
    const monitor = await currentMonitor();
    const target = monitor
      ? rosterOrigin(
          position,
          { width: ROSTER_WIDTH, height },
          {
            x: monitor.workArea.position.x,
            y: monitor.workArea.position.y,
            width: monitor.workArea.size.width,
            height: monitor.workArea.size.height,
          },
          monitor.scaleFactor,
        )
      : position;
    await win.setSize(new LogicalSize(ROSTER_WIDTH, height));
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
