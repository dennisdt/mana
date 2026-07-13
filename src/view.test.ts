import { describe, expect, it } from "vitest";
import { cardHtml, pillHtml, type Snapshot } from "./view";

const weeklyOnly: Snapshot = {
  provider: "codex",
  plan: "pro",
  status: "ok",
  fetched_at: 1_784_487_600,
  bars: [{ id: "weekly", label: "Weekly", used_percent: 55, resets_at: 1_784_487_600 }],
};

describe("cardHtml", () => {
  it("renders a provider-owned familiar and weekly-only row", () => {
    const html = cardHtml(weeklyOnly, "codex");
    expect(html).toContain('class="sprite nimbus"');
    expect(html).toContain('data-provider="codex"');
    expect(html).toContain('class="lbl">Weekly</span>');
    expect(html).not.toContain("5 hour");
  });

  it("keeps the provider roster shell when data is absent", () => {
    const html = cardHtml(undefined, "claude");
    expect(html).toContain('class="sprite clawd"');
    expect(html).toContain("Claude");
    expect(html).toContain("log in via the claude CLI");
  });

  it("escapes limit labels", () => {
    const snapshot = { ...weeklyOnly, bars: [{ ...weeklyOnly.bars[0], label: '<script>' }] };
    expect(cardHtml(snapshot, "codex")).toContain("&#60;script&#62;");
  });
});

describe("pillHtml", () => {
  it("uses the first available bar when no session bar exists", () => {
    expect(pillHtml(weeklyOnly, "codex")).toContain('data-bar="0"');
  });
});
