import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { fmtAge, fmtCountdown, manaLeft } from "./format";

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

function barHtml(s: Snapshot, bar: Bar): string {
  const left = manaLeft(bar.used_percent);
  const low = left < 30 ? " low" : "";
  return `<div class="track ${s.provider}${low}"><div class="fill" style="width:${left}%"></div></div>`;
}

function pillHtml(s: Snapshot | undefined, provider: string): string {
  if (!s || s.status === "absent" || s.bars.length === 0) {
    return `<span class="gem">${GEMS[provider]}</span><span class="nums">no data</span>`;
  }
  const session = s.bars.find((b) => b.id === "session") ?? s.bars[0];
  const left = manaLeft(session.used_percent);
  const cd = fmtCountdown(session.resets_at, Date.now());
  const stale = s.status === "stale" ? " stale" : "";
  return `<span class="gem ${s.provider}${stale}">${GEMS[provider]}</span>
    ${barHtml(s, session)}
    <span class="nums${stale}"><b>${Math.round(left)}%</b>${cd ? " · " + cd : ""}</span>`;
}

function cardHtml(s: Snapshot | undefined, provider: string): string {
  const name = provider === "claude" ? "Claude" : "Codex";
  if (!s || s.status === "absent" || s.bars.length === 0) {
    return `<div class="head">${name}</div><div class="empty">no data — log in via the ${provider} CLI</div>`;
  }
  const stale = s.status === "stale" ? " stale" : "";
  const age = s.status === "stale" ? `<span class="age">${fmtAge(s.fetched_at, Date.now())}</span>` : "";
  const plan = s.plan ? `<span class="plan">${s.plan}</span>` : "";
  const rows = s.bars
    .map((b) => {
      const left = manaLeft(b.used_percent);
      const cd = fmtCountdown(b.resets_at, Date.now());
      return `<div class="row${stale}">
        <span class="lbl">${b.label}</span>
        ${barHtml(s, b)}
        <span class="val"><b>${Math.round(left)}%</b>${cd ? " · " + cd : ""}</span>
      </div>`;
    })
    .join("");
  return `<div class="head">${name}${plan}${age}</div>${rows}`;
}

function render(): void {
  for (const provider of ["claude", "codex"]) {
    const s = snapshots.get(provider);
    document.getElementById(`pill-${provider}`)!.innerHTML = pillHtml(s, provider);
    document.getElementById(`card-${provider}`)!.innerHTML = cardHtml(s, provider);
  }
}

async function expand(): Promise<void> {
  await getCurrentWindow().setSize(EXPANDED);
  document.body.classList.add("expanded");
}

async function collapse(): Promise<void> {
  document.body.classList.remove("expanded");
  await getCurrentWindow().setSize(COLLAPSED);
}

document.body.addEventListener("mouseenter", () => void expand());
document.body.addEventListener("mouseleave", () => void collapse());

void listen<Snapshot>("usage-update", (e) => {
  snapshots.set(e.payload.provider, e.payload);
  render();
});

void invoke<Snapshot[]>("get_snapshots").then((all) => {
  for (const s of all) snapshots.set(s.provider, s);
  render();
});

setInterval(render, 1000);
