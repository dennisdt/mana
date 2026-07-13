import { manaLeft } from "./format";

export type Bar = {
  id: string;
  label: string;
  used_percent: number;
  resets_at: number | null;
};

export type Snapshot = {
  provider: string;
  bars: Bar[];
  plan: string | null;
  status: string;
  fetched_at: number;
};

const GEMS: Record<string, string> = { claude: "◆", codex: "●" };

function esc(value: string): string {
  return value.replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);
}

function spriteHtml(provider: string): string {
  const className = provider === "claude" ? "clawd" : "nimbus";
  return `<div class="sprite ${className}" data-provider="${provider}" data-state="idle" aria-hidden="true"></div>`;
}

function barHtml(snapshot: Snapshot, bar: Bar, index: number): string {
  const left = manaLeft(bar.used_percent);
  return `<div class="track ${snapshot.provider}${left < 30 ? " low" : ""}" data-bar="${index}"><div class="fill" style="width:${left}%"></div></div>`;
}

export function pillHtml(snapshot: Snapshot | undefined, provider: string): string {
  if (!snapshot || snapshot.status === "absent" || snapshot.bars.length === 0) {
    return `<span class="gem">${GEMS[provider]}</span><span class="nums">no data</span>`;
  }
  const sessionIndex = snapshot.bars.findIndex((bar) => bar.id === "session");
  const index = sessionIndex >= 0 ? sessionIndex : 0;
  return `<span class="gem ${snapshot.provider}">${GEMS[provider]}</span>
    ${barHtml(snapshot, snapshot.bars[index], index)}
    <span class="nums"><b class="pct" data-bar="${index}"></b><span class="cd" data-bar="${index}"></span></span>`;
}

export function cardHtml(snapshot: Snapshot | undefined, provider: string): string {
  const name = provider === "claude" ? "Claude" : "Codex";
  const hasData = snapshot && snapshot.status !== "absent" && snapshot.bars.length > 0;
  const rows = hasData
    ? `<div class="rows">${snapshot.bars.map((bar, index) => `<div class="row">
        <span class="lbl">${esc(bar.label)}</span>
        ${barHtml(snapshot, bar, index)}
        <span class="val"><b class="pct" data-bar="${index}"></b><span class="cd" data-bar="${index}"></span></span>
      </div>`).join("")}</div>`
    : `<div class="empty">no data - log in via the ${provider} CLI</div>`;
  return `<div class="familiar-slot">${spriteHtml(provider)}</div>
    <div class="provider-content">
      <div class="head"><strong>${name}</strong><span class="plan"></span><span class="activity-signal" aria-hidden="true"></span><span class="age"></span></div>
      ${rows}
    </div>`;
}
