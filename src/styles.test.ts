// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const styles = readFileSync(new URL("./styles.css", import.meta.url), "utf8");

describe("fantasy gaming HUD stylesheet", () => {
  it("uses only the original generated frame", () => {
    expect(styles).toContain('url("/hud/mana-bar-frame.png")');
    expect(styles).not.toContain("boss-bar-");
    expect(styles).not.toContain("background-size: 4px 4px");
    expect(styles).not.toMatch(/\.track::after\s*\{[^}]*mana-bar-frame/s);
  });

  it("renders free-standing illustrated mage atlases", () => {
    expect(styles).toContain('url("/sprites/claude-fire-poison.png")');
    expect(styles).toContain('url("/sprites/codex-ice-lightning.png")');
    expect(styles).not.toContain("clawd.png");
    expect(styles).not.toContain("nimbus.png");
    expect(styles).not.toContain(".familiar-slot::before");
    expect(styles).not.toContain(".familiar-slot::after");
    expect(styles).not.toContain("image-rendering: pixelated");
    expect(styles).toMatch(/#card section\s*\{[^}]*grid-template-columns:\s*60px minmax\(0, 1fr\)/s);
    const spriteRule = styles.match(/\.sprite\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(spriteRule).toMatch(/width:\s*56px/);
    expect(spriteRule).toMatch(/height:\s*56px/);
    expect(spriteRule).toMatch(/background-size:\s*224px 168px/);
    expect(spriteRule).not.toContain("animation:");
    expect(styles).toMatch(/\.sprite\[data-frame="0"\]\s*\{[^}]*background-position-x:\s*0/s);
    expect(styles).toMatch(/\.sprite\[data-frame="1"\]\s*\{[^}]*background-position-x:\s*-56px/s);
    expect(styles).toMatch(/\.sprite\[data-frame="2"\]\s*\{[^}]*background-position-x:\s*-112px/s);
    expect(styles).toMatch(/\.sprite\[data-frame="3"\]\s*\{[^}]*background-position-x:\s*-168px/s);
    expect(styles).toMatch(/\.sprite\[data-state="working"\]\s*\{[^}]*background-position-y:\s*-56px/s);
    expect(styles).toMatch(/\.sprite\[data-state="hover"\]\s*\{[^}]*background-position-y:\s*-112px/s);
    expect(styles).not.toContain("sprite-run");
  });

  it("declares fixed frame and live-core geometry", () => {
    expect(styles).toContain("--meter-width: 144px");
    expect(styles).toContain("--meter-height: 20px");
    expect(styles).toContain("--meter-inset-x: 14px");
    expect(styles).toContain("--meter-inset-y: 6px");
    expect(styles).toContain("--meter-channel-width: 116px");
    expect(styles).toContain("--meter-channel-height: 8px");
    expect(styles).toMatch(/\.fill\s*\{[^}]*top:\s*var\(--meter-inset-y\)[^}]*left:\s*var\(--meter-inset-x\)[^}]*max-width:\s*var\(--meter-channel-width\)/s);
    expect(styles).toContain('.fill[data-empty="true"]');
  });

  it("keeps stale text readable and covers reduced motion", () => {
    expect(styles).not.toMatch(/\.stale\s*\{[^}]*\bfilter\s*:/s);
    expect(styles).not.toMatch(/\.stale\s*\{[^}]*\bopacity\s*:/s);
    expect(styles).not.toContain(".stale .sprite");
    expect(styles).toContain(".stale .fill");
    expect(styles).toContain("animation: magic-glint 3.2s ease-in-out infinite");
    expect(styles).toContain("@media (prefers-reduced-motion: reduce)");
    const reducedMotion = styles.slice(
      styles.indexOf("@media (prefers-reduced-motion: reduce)"),
    );
    expect(reducedMotion).toContain(".sprite[data-frame]");
    expect(reducedMotion).toContain("background-position-x: 0");
    expect(reducedMotion).toContain(".fill::before");
    expect(reducedMotion).toContain(".provider-card[data-working] .activity-signal");
    expect(reducedMotion).toContain("animation: none");
  });

  it("staggers provider pulses and row glints with deterministic offsets", () => {
    expect(styles).toMatch(/#card-claude\s*\{[^}]*--provider-motion-offset:\s*0s/s);
    expect(styles).toMatch(/#card-codex\s*\{[^}]*--provider-motion-offset:\s*-0\.8s/s);
    expect(styles).toMatch(/\.row:nth-child\(1\)\s*\{[^}]*--row-motion-offset:\s*0s/s);
    expect(styles).toMatch(/\.row:nth-child\(2\)\s*\{[^}]*--row-motion-offset:\s*-0\.85s/s);
    expect(styles).toMatch(/\.row:nth-child\(3\)\s*\{[^}]*--row-motion-offset:\s*-1\.7s/s);
    expect(styles).toMatch(/\.row:nth-child\(4\)\s*\{[^}]*--row-motion-offset:\s*-2\.55s/s);
    expect(styles).toMatch(
      /\.fill::before\s*\{[^}]*animation-delay:\s*calc\(var\(--provider-motion-offset\) \+ var\(--row-motion-offset\)\)/s,
    );
    expect(styles).toMatch(
      /\.provider-card\[data-working\] \.activity-signal\s*\{[^}]*animation-delay:\s*var\(--provider-motion-offset\)/s,
    );
  });

  it("preserves intrinsic card height for native content measurement", () => {
    expect(styles).toMatch(/#root\s*\{[^}]*flex-direction:\s*column/s);
    expect(styles).not.toMatch(/#card\s*\{[^}]*\bflex:\s*1(?:\s|;)/s);
  });
});
