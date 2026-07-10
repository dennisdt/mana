import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { fmtAge, fmtCountdown, manaLeft, planLabel } from "./format";

type Bar = {
  id: string;
  label: string;
  used_percent: number;
  resets_at: number | null;
};
type Snapshot = {
  provider: string;
  bars: Bar[];
  plan: string | null;
  status: string;
  fetched_at: number;
};

const COLLAPSED = new LogicalSize(280, 44);
const EXPANDED = new LogicalSize(300, 248);
const GEMS: Record<string, string> = { claude: "◆", codex: "●" };

const snapshots = new Map<string, Snapshot>();

function esc(s: string): string {
  return s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

function barHtml(s: Snapshot, bar: Bar, idx: number): string {
  const left = manaLeft(bar.used_percent);
  const low = left < 30 ? " low" : "";
  return `<div class="track ${s.provider}${low}" data-bar="${idx}"><div class="fill" style="width:${left}%"></div></div>`;
}

function pillHtml(s: Snapshot | undefined, provider: string): string {
  if (!s || s.status === "absent" || s.bars.length === 0) {
    return `<span class="gem">${GEMS[provider]}</span><span class="nums">no data</span>`;
  }
  const found = s.bars.findIndex((b) => b.id === "session");
  const idx = found >= 0 ? found : 0;
  const session = s.bars[idx];
  return `<span class="gem ${s.provider}">${GEMS[provider]}</span>
    ${barHtml(s, session, idx)}
    <span class="nums"><b class="pct" data-bar="${idx}"></b><span class="cd" data-bar="${idx}"></span></span>`;
}

function cardHtml(s: Snapshot | undefined, provider: string): string {
  const name = provider === "claude" ? "Claude" : "Codex";
  if (!s || s.status === "absent" || s.bars.length === 0) {
    return `<div class="head">${name}</div><div class="empty">no data — log in via the ${provider} CLI</div>`;
  }
  const rows = s.bars
    .map(
      (b, i) => `<div class="row">
        <span class="lbl">${esc(b.label)}</span>
        ${barHtml(s, b, i)}
        <span class="val"><b class="pct" data-bar="${i}"></b><span class="cd" data-bar="${i}"></span></span>
      </div>`,
    )
    .join("");
  return `<div class="head">${name}<span class="plan"></span><span class="age"></span></div>${rows}`;
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
      ? s.bars.map((b) => b.id).join(",")
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

let sizing: Promise<void> = Promise.resolve();
function setExpanded(on: boolean): void {
  sizing = sizing.then(async () => {
    if (on) {
      await getCurrentWindow().setSize(EXPANDED);
      document.body.classList.add("expanded");
    } else {
      document.body.classList.remove("expanded");
      await getCurrentWindow().setSize(COLLAPSED);
    }
  });
}

document.body.addEventListener("mouseenter", () => setExpanded(true));
document.body.addEventListener("mouseleave", () => setExpanded(false));

void listen<Snapshot>("usage-update", (e) => {
  snapshots.set(e.payload.provider, e.payload);
  renderProvider(e.payload.provider);
});

void invoke<Snapshot[]>("get_snapshots").then((all) => {
  for (const s of all) snapshots.set(s.provider, s);
  for (const provider of ["claude", "codex"]) renderProvider(provider);
});

for (const provider of ["claude", "codex"]) renderProvider(provider);

setInterval(tick, 1000);
