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
