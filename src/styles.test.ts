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
    expect(styles).toMatch(/\.sprite\s*\{[^}]*width:\s*56px[^}]*height:\s*56px[^}]*background-size:\s*224px 168px[^}]*animation:\s*sprite-run 1\.15s steps\(4\) infinite/s);
    expect(styles).toMatch(/\.sprite\[data-state="working"\]\s*\{[^}]*background-position-y:\s*-56px[^}]*animation-duration:\s*0\.68s/s);
    expect(styles).toMatch(/\.sprite\[data-state="hover"\]\s*\{[^}]*background-position-y:\s*-112px[^}]*animation-duration:\s*0\.82s/s);
    expect(styles).toMatch(/@keyframes sprite-run\s*\{[^}]*background-position-x:\s*0[^}]*\}[^}]*background-position-x:\s*-224px/s);
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
    expect(reducedMotion).toContain(".sprite");
    expect(reducedMotion).toContain(".fill::before");
    expect(reducedMotion).toContain(".provider-card[data-working] .activity-signal");
    expect(reducedMotion).toContain("animation: none");
  });

  it("preserves intrinsic card height for native content measurement", () => {
    expect(styles).toMatch(/#root\s*\{[^}]*flex-direction:\s*column/s);
    expect(styles).not.toMatch(/#card\s*\{[^}]*\bflex:\s*1(?:\s|;)/s);
  });
});
