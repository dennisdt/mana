import { describe, expect, it } from "vitest";
import indexMarkup from "../index.html?raw";
import { cardHtml, providerIsVisible } from "./view";

const weeklyOnly = {
  provider: "codex",
  plan: "pro",
  status: "ok",
  fetched_at: 1_784_487_600,
  bars: [{ id: "weekly", label: "Weekly", used_percent: 55, resets_at: 1_784_487_600 }],
};

describe("cardHtml", () => {
  it("renders a provider-owned familiar and weekly-only row", () => {
    const html = cardHtml(weeklyOnly, "codex");
    expect(html).toContain(
      '<div class="aura" data-provider="codex" data-frame="0" aria-hidden="true"></div>',
    );
    expect(html).toContain('class="sprite codex-mage"');
    expect(html.indexOf('class="aura"')).toBeLessThan(html.indexOf('class="sprite'));
    expect(html).toContain('data-provider="codex"');
    expect(html).toContain('data-state="idle" data-frame="0"');
    expect(html).toContain('class="lbl">Weekly</span>');
    expect(html).toContain('class="track codex"');
    expect(html).toContain('class="fill" data-empty="false" style="width:52px"');
    expect(html.match(/class="row"/g)).toHaveLength(1);
    expect(html).not.toContain('class="slot"');
    expect(html).not.toContain("5 hour");
  });

  it("marks a depleted meter so zero width cannot retain a glow", () => {
    const depleted = {
      ...weeklyOnly,
      bars: [{ ...weeklyOnly.bars[0], used_percent: 100 }],
    };
    const html = cardHtml(depleted, "codex");
    expect(html).toContain('class="fill" data-empty="true" style="width:0px"');
  });

  it("keeps the provider roster shell when data is absent", () => {
    const html = cardHtml(undefined, "claude");
    expect(html).toContain('class="aura"');
    expect(html).toContain('class="sprite claude-mage"');
    expect(html).toContain('data-state="idle" data-frame="0"');
    expect(html).toContain("Claude");
    expect(html).toContain("log in via the claude CLI");
  });

  it("omits the superseded activity indicator", () => {
    expect(cardHtml(weeklyOnly, "claude")).not.toContain("activity-signal");
  });

  it("returns no provider or aura markup when explicitly unauthenticated", () => {
    const snapshot = { ...weeklyOnly, authenticated: false };
    expect(cardHtml(snapshot, "codex")).toBe("");
    expect(cardHtml({ ...snapshot, provider: "claude" }, "claude")).toBe("");
  });

  it("escapes limit labels", () => {
    const snapshot = { ...weeklyOnly, bars: [{ ...weeklyOnly.bars[0], label: '<script>' }] };
    expect(cardHtml(snapshot, "codex")).toContain("&#60;script&#62;");
  });
});

describe("providerIsVisible", () => {
  it("hides only explicitly unauthenticated providers", () => {
    expect(providerIsVisible({ ...weeklyOnly, authenticated: false })).toBe(false);
    expect(providerIsVisible({ ...weeklyOnly, authenticated: true, status: "absent", bars: [] })).toBe(true);
  });

  it("keeps startup and legacy snapshots visible", () => {
    expect(providerIsVisible(undefined)).toBe(true);
    expect(providerIsVisible(weeklyOnly)).toBe(true);
  });
});

it("mounts the roster without a compact pill", () => {
  expect(indexMarkup).toContain('id="card"');
  expect(indexMarkup).not.toContain('id="pill"');
});
