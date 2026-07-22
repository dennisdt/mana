// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const styles = readFileSync(new URL("./styles.css", import.meta.url), "utf8");
const mainSource = readFileSync(new URL("./main.ts", import.meta.url), "utf8");
const librs = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

describe("fantasy gaming HUD stylesheet", () => {
  it("keeps the original generated frame as the root fallback", () => {
    expect(styles).toMatch(
      /:root\s*\{[^}]*--meter-frame-art:\s*url\("\/hud\/mana-bar-frame\.png"\)/s,
    );
    expect(styles).not.toContain("boss-bar-");
    expect(styles).not.toContain("background-size: 4px 4px");
  });

  it("renders free-standing illustrated mage atlases", () => {
    expect(styles).toContain('url("/sprites/claude-fire-poison.png")');
    expect(styles).toContain('url("/sprites/codex-ice-lightning.png")');
    expect(styles).not.toContain("clawd.png");
    expect(styles).not.toContain("nimbus.png");
    expect(styles).toContain(".familiar-slot::before");
    expect(styles).not.toContain(".familiar-slot::after");
    expect(styles).toMatch(/#card section\s*\{[^}]*grid-template-columns:\s*70px minmax\(0, 1fr\)/s);
    const spriteRule = styles.match(/\.sprite\s*\{([^}]*)\}/s)?.[1] ?? "";
    const spriteArtRule = styles.match(/\.sprite::before\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(spriteRule).toMatch(/width:\s*56px/);
    expect(spriteRule).toMatch(/height:\s*56px/);
    expect(spriteArtRule).toMatch(/background-size:\s*272px 204px/);
    expect(spriteArtRule).toMatch(/image-rendering:\s*auto/);
    expect(spriteArtRule).not.toMatch(/drop-shadow\(0 0/);
    expect(spriteRule).not.toContain("animation:");
    expect(styles).toMatch(/\.sprite\[data-frame="0"\]::before\s*\{[^}]*background-position-x:\s*0/s);
    expect(styles).toMatch(/\.sprite\[data-frame="1"\]::before\s*\{[^}]*background-position-x:\s*-68px/s);
    expect(styles).toMatch(/\.sprite\[data-frame="2"\]::before\s*\{[^}]*background-position-x:\s*-136px/s);
    expect(styles).toMatch(/\.sprite\[data-frame="3"\]::before\s*\{[^}]*background-position-x:\s*-204px/s);
    expect(styles).toMatch(/\.sprite\[data-state="working"\]::before\s*\{[^}]*background-position-y:\s*-68px/s);
    expect(styles).toMatch(/\.sprite\[data-state="hover"\]::before\s*\{[^}]*background-position-y:\s*-136px/s);
    expect(styles).not.toContain("sprite-run");
  });

  it("counter-scales two-times sprite art against the fixed widget zoom", () => {
    expect(mainSource).toMatch(
      /document\.documentElement\.style\.setProperty\(\s*"--sprite-resolution-scale",\s*String\(1 \/ WIDGET_ZOOM\),?\s*\)/s,
    );
    const spriteRule = styles.match(/\.sprite\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(spriteRule).toMatch(/zoom:\s*var\(--sprite-resolution-scale,\s*1\)/);
    expect(spriteRule).not.toMatch(/\bfilter\s*:/);
  });

  it("renders each atlas frame on an oversized layer that can overflow its anchor", () => {
    const spriteRule = styles.match(/\.sprite\s*\{([^}]*)\}/s)?.[1] ?? "";
    const artRule = styles.match(/\.sprite::before\s*\{([^}]*)\}/s)?.[1] ?? "";

    expect(spriteRule).toMatch(/width:\s*56px/);
    expect(spriteRule).toMatch(/height:\s*56px/);
    expect(spriteRule).toMatch(/overflow:\s*visible/);
    expect(spriteRule).not.toMatch(/background-(?:image|size|position|repeat)/);
    expect(artRule).toMatch(/position:\s*absolute/);
    expect(artRule).toMatch(/width:\s*68px/);
    expect(artRule).toMatch(/height:\s*68px/);
    expect(artRule).toMatch(/top:\s*50%/);
    expect(artRule).toMatch(/left:\s*50%/);
    expect(artRule).toMatch(/transform:\s*translate\(-50%,\s*-50%\)/);
    expect(artRule).toMatch(/background-image:\s*var\(--sprite-sheet\)/);
    expect(artRule).toMatch(/background-size:\s*272px 204px/);
    expect(styles).toMatch(
      /\.sprite\[data-frame="1"\]::before\s*\{[^}]*background-position-x:\s*-68px/s,
    );
    expect(styles).toMatch(
      /\.sprite\[data-state="hover"\]::before\s*\{[^}]*background-position-y:\s*-136px/s,
    );
    expect(styles).toMatch(
      /\.sprite\.codex-mage\s*\{[^}]*--sprite-sheet:\s*url\("\/sprites\/codex-ice-lightning\.png"\)/s,
    );
    expect(mainSource).toContain(
      'element.style.setProperty("--sprite-sheet", `url("${url}")`)',
    );
    expect(mainSource).not.toContain("element.style.backgroundImage");
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
    expect(reducedMotion).toContain("#progress .xpfill::before");
    expect(reducedMotion).toContain("#progress .xpfill::after");
    expect(reducedMotion).toContain("animation: none");
  });

  it("staggers row glints with deterministic provider and row offsets", () => {
    expect(styles).toMatch(/#card-claude\s*\{[^}]*--provider-motion-offset:\s*0s/s);
    expect(styles).toMatch(/#card-codex\s*\{[^}]*--provider-motion-offset:\s*-0\.8s/s);
    expect(styles).toMatch(/\.row:nth-child\(1\)\s*\{[^}]*--row-motion-offset:\s*0s/s);
    expect(styles).toMatch(/\.row:nth-child\(2\)\s*\{[^}]*--row-motion-offset:\s*-0\.85s/s);
    expect(styles).toMatch(/\.row:nth-child\(3\)\s*\{[^}]*--row-motion-offset:\s*-1\.7s/s);
    expect(styles).toMatch(/\.row:nth-child\(4\)\s*\{[^}]*--row-motion-offset:\s*-2\.55s/s);
    expect(styles).toMatch(
      /\.fill::before\s*\{[^}]*animation-delay:\s*calc\(var\(--provider-motion-offset\) \+ var\(--row-motion-offset\)\)/s,
    );
  });

  it("aligns percentages independently from reset-time length", () => {
    expect(styles).toMatch(
      /\.row \.val\s*\{[^}]*display:\s*grid[^}]*grid-template-columns:\s*4ch minmax\(0, 1fr\)[^}]*column-gap:\s*8px/s,
    );
    expect(styles).toMatch(/\.row \.pct\s*\{[^}]*text-align:\s*right/s);
    expect(styles).toMatch(/\.row \.cd\s*\{[^}]*text-align:\s*left/s);
  });

  it("preserves intrinsic card height for native content measurement", () => {
    expect(styles).toMatch(/#root\s*\{[^}]*flex-direction:\s*column/s);
    expect(styles).not.toMatch(/#card\s*\{[^}]*\bflex:\s*1(?:\s|;)/s);
  });

  it("keeps hidden provider sections out of the grid layout", () => {
    expect(styles).toMatch(
      /#card section\[hidden\]\s*\{[^}]*display:\s*none/s,
    );
  });

  it("adds provider divider spacing only between visible sections", () => {
    expect(styles).toMatch(
      /#card section:not\(\[hidden\]\) \+ section:not\(\[hidden\]\)\s*\{[^}]*padding-top:\s*14px[^}]*border-top:\s*1px solid var\(--line\)/s,
    );
    expect(styles).not.toMatch(/#card section \+ section\s*\{/);
  });
});

describe("concentric nested corner radii", () => {
  it("keeps the vibrancy blur radius equal to the CSS --hud-radius", () => {
    const cssRadius = styles.match(/--hud-radius:\s*(\d+(?:\.\d+)?)px/)?.[1];
    const rustRadius = librs.match(/HUD_CORNER_RADIUS:\s*f64\s*=\s*(\d+(?:\.\d+)?)/)?.[1];
    expect(cssRadius).toBeDefined();
    expect(rustRadius).toBeDefined();
    expect(Number(rustRadius)).toBe(Number(cssRadius));
    expect(librs).toMatch(/apply_vibrancy\([^;]*Some\(HUD_CORNER_RADIUS\),\s*\)\?;/s);
  });

  it("derives the corner tick radius as HUD radius minus tick inset", () => {
    expect(styles).toContain("--corner-tick-inset: 5px");
    expect(styles).toContain(
      "--corner-tick-radius: max(0px, calc(var(--hud-radius) - var(--corner-tick-inset)))",
    );
    expect(styles).toMatch(
      /#root::before\s*\{[^}]*inset:\s*var\(--corner-tick-inset\)[^}]*border-radius:\s*var\(--corner-tick-radius\)/s,
    );
  });

  it("derives the fill radius as meter frame radius minus its vertical inset", () => {
    expect(styles).toContain("--meter-frame-radius: var(--hud-radius)");
    expect(styles).toContain(
      "--fill-radius: max(1px, calc(var(--meter-frame-radius) - var(--meter-inset-y)))",
    );
    expect(styles).toMatch(/\.fill\s*\{[^}]*border-radius:\s*var\(--fill-radius\)/s);
    const fill = styles.match(/\.fill\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(fill).not.toMatch(/border-radius:\s*\d/);
  });
});

describe("rank border themes", () => {
  const tiers: Array<[string, string, string, string]> = [
    ["plastic", "#b8bec8", "#8b929e", "transparent"],
    ["wood", "#a5713d", "#6b4726", "transparent"],
    ["iron", "#9aa3ad", "#5f6770", "rgba(154, 163, 173, 0.25)"],
    ["bronze", "#cd7f32", "#8c5a24", "rgba(205, 127, 50, 0.3)"],
    ["silver", "#e6edf5", "#97a3b4", "rgba(230, 237, 245, 0.35)"],
    ["gold", "#f2c968", "#b8862e", "rgba(242, 201, 104, 0.45)"],
    ["platinum", "#dfe9ec", "#9fb6c4", "rgba(223, 233, 236, 0.5)"],
    ["emerald", "#3ddc84", "#147a4a", "rgba(61, 220, 132, 0.5)"],
    ["diamond", "#9be8ff", "#4aa8d8", "rgba(155, 232, 255, 0.55)"],
    ["master", "#ff5a6e", "#a3172c", "rgba(255, 90, 110, 0.55)"],
    ["legend", "#b06aff", "#5e1ea8", "rgba(176, 106, 255, 0.55)"],
    ["champion", "#ffd75e", "#3f8cff", "rgba(255, 215, 94, 0.6)"],
    ["godlike", "#fff6d8", "#ffd9f6", "rgba(255, 246, 216, 0.75)"],
  ];

  it("themes every dressed tier with frame custom properties", () => {
    for (const [tier, frame1, frame2, glow] of tiers) {
      const rule =
        styles.match(new RegExp(`#root\\[data-rank="${tier}"\\][^{]*\\{([^}]*)\\}`, "s"))?.[1] ??
        "";
      expect(rule, tier).toContain(`--frame-1: ${frame1}`);
      expect(rule, tier).toContain(`--frame-2: ${frame2}`);
      expect(rule, tier).toContain(`--frame-glow: ${glow}`);
    }
  });

  it("consumes the frame variables from the original shared border rule", () => {
    expect(styles).toMatch(
      /#root\s*\{[^}]*border:\s*1px solid var\(--frame-1, rgba\(205, 221, 242, 0\.34\)\)/s,
    );
    expect(styles).toMatch(/#root\s*\{[^}]*0 0 14px var\(--frame-glow, transparent\)/s);
    expect(styles).toMatch(/#root\s*\{[^}]*0 0 30px var\(--frame-glow-2, transparent\)/s);
  });

  it("keeps naked and unranked roots borderless without corner ticks", () => {
    expect(styles).toMatch(
      /#root:not\(\[data-rank\]\)[^{]*\{[^}]*border-color:\s*transparent/s,
    );
    expect(styles).toMatch(/#root\[data-rank="naked"\][^{]*\{[^}]*border-color:\s*transparent/s);
    expect(styles).toMatch(/#root:not\(\[data-rank\]\)::before[^{]*\{[^}]*content:\s*none/s);
    expect(styles).toMatch(/#root\[data-rank="naked"\]::before[^{]*\{[^}]*content:\s*none/s);
  });

  it("rings every dressed tier with the original radius-following gradient", () => {
    expect(styles).not.toMatch(/border-image\s*:/);
    expect(styles).toMatch(/#frame\s*\{[^}]*border-radius:\s*var\(--hud-radius\)/s);
    expect(styles).toMatch(/#frame\s*\{[^}]*-webkit-mask-composite:\s*xor/s);
    expect(styles).toMatch(/#frame\s*\{[^}]*pointer-events:\s*none/s);
    expect(styles).toMatch(/#frame\s*\{[^}]*background:\s*var\(--ring, none\)/s);
    expect(styles).toMatch(
      /#frame\s*\{[^}]*border:\s*var\(--frame-w, 1px\) solid transparent/s,
    );
    for (const [tier] of tiers) {
      expect(styles, tier).toMatch(
        new RegExp(`#root\\[data-rank="${tier}"\\][^{]*\\{[^}]*--ring:\\s*linear-gradient\\(`, "s"),
      );
    }
  });

  it("escalates the original frame weight with rank", () => {
    const weights: Array<[string, string]> = [
      ["plastic", "1px"], ["wood", "1px"],
      ["iron", "2px"], ["bronze", "2px"], ["silver", "2px"], ["gold", "2px"],
      ["platinum", "2px"], ["emerald", "2px"], ["diamond", "2px"],
      ["master", "3px"], ["legend", "3px"], ["champion", "3px"], ["godlike", "3px"],
    ];
    for (const [tier, weight] of weights) {
      expect(styles, tier).toMatch(
        new RegExp(`#root\\[data-rank="${tier}"\\][^{]*\\{[^}]*--frame-w:\\s*${weight}`, "s"),
      );
    }
  });

  it("tints the original corner ticks from the frame on the top tiers", () => {
    expect(styles).toMatch(
      /#root::before[^{]*\{[^}]*var\(--tick-1, rgba\(219, 231, 244, 0\.58\)\)/s,
    );
    expect(styles).toMatch(
      /#root::before[^{]*\{[^}]*var\(--tick-2, rgba\(242, 201, 104, 0\.52\)\)/s,
    );
    for (const tier of ["master", "legend", "champion", "godlike"]) {
      expect(styles, tier).toMatch(
        new RegExp(`#root\\[data-rank="${tier}"\\][^{]*\\{[^}]*--tick-1:`, "s"),
      );
    }
  });

  it("restores Champion and Godlike motion with reduced-motion coverage", () => {
    expect(styles).toContain("@keyframes godlike-halo");
    expect(styles).toContain("@keyframes champion-radiance");
    expect(styles).toMatch(
      /#root\[data-rank="godlike"\]\s*\{[^}]*animation:\s*godlike-halo/s,
    );
    expect(styles).toMatch(
      /#root\[data-rank="champion"\] #frame\s*\{[^}]*animation:\s*champion-radiance 6s linear infinite/s,
    );
    const reducedMotion = styles.slice(
      styles.indexOf("@media (prefers-reduced-motion: reduce)"),
    );
    expect(reducedMotion).toContain('#root[data-rank="champion"]');
    expect(reducedMotion).toContain('#root[data-rank="godlike"]');
  });

  it("restores the original Godlike second outer glow", () => {
    expect(styles).toMatch(
      /#root\[data-rank="godlike"\][^{]*\{[^}]*--frame-glow-2:\s*rgba\(190, 225, 255, 0\.4\)/s,
    );
  });
});

describe("prestige badges", () => {
  it("sizes badges and maps each slot to its art", () => {
    const badge = styles.match(/\.badge\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(badge).toMatch(/width:\s*24px/);
    expect(badge).toMatch(/height:\s*24px/);
    for (let n = 1; n <= 10; n += 1) {
      expect(styles).toContain(`url("/badges/prestige-${n}.png")`);
    }
  });

  it("renders the overflow count as a superscript on the tenth badge", () => {
    expect(styles).toMatch(
      /\.badge\[data-count\]::before\s*\{[^}]*content:\s*attr\(data-count\)/s,
    );
  });

  it("uses an empty-content circular crest when badge art is missing", () => {
    expect(styles).toMatch(
      /\.badge\[data-fallback="true"\]::after\s*\{[^}]*content:\s*""[^}]*border-radius:\s*50%[^}]*radial-gradient/s,
    );
    expect(styles).not.toContain("★");
  });
});

describe("rank armor art integration", () => {
  const tiers = [
    "naked", "plastic", "wood", "iron", "bronze", "silver", "gold",
    "platinum", "emerald", "diamond", "master", "legend", "champion", "godlike",
  ];

  it("maps every known rank to exactly one matching meter foreground", () => {
    for (const tier of tiers) {
      expect(styles, tier).toMatch(
        new RegExp(
          `#root\\[data-rank="${tier}"\\][^{]*\\{[^}]*--meter-frame-art:\\s*url\\("/hud/mana-bar-frame-${tier}\\.png"\\)`,
          "s",
        ),
      );
    }
  });

  it("renders only the selected frame in a foreground above the live fill", () => {
    const track = styles.match(/\.track\s*\{([^}]*)\}/s)?.[1] ?? "";
    const foreground = styles.match(/\.track::after\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(track).toMatch(/background:\s*none/);
    expect(track).not.toContain("url(");
    expect(foreground).toMatch(/position:\s*absolute/);
    expect(foreground).toMatch(/inset:\s*0/);
    expect(foreground).toMatch(/z-index:\s*2/);
    expect(foreground).toMatch(/background-image:\s*var\(--meter-frame-art\)/);
    expect(foreground).not.toContain('mana-bar-frame.png');
  });

  it("keeps the action clickable in its angular top-right corner", () => {
    expect(styles).toMatch(
      /#action\s*\{[^}]*top:\s*4px[^}]*right:\s*4px[^}]*clip-path:[^}]*-webkit-app-region:\s*no-drag/s,
    );
  });

  it("uses original frame tokens for the action without armor-shell tokens", () => {
    const action = styles.match(/#action\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(action).toMatch(/var\(--frame-1/);
    expect(action).toMatch(/var\(--frame-2/);
    expect(action).not.toContain("--armor-");
    expect(styles).not.toContain("--armor-");
  });

  it("moves working feedback from diamonds to a radial glow behind the familiar", () => {
    expect(styles).not.toContain("activity-signal");
    expect(styles).toMatch(
      /\.familiar-slot::before\s*\{[^}]*z-index:\s*1[^}]*top:\s*50%[^}]*left:\s*50%[^}]*width:\s*72px[^}]*height:\s*72px[^}]*radial-gradient\(\s*circle[^}]*var\(--glow\)[^}]*pointer-events:\s*none/s,
    );
    const glow = styles.match(/\.familiar-slot::before\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(glow).not.toMatch(/\bfilter\s*:/);
    expect(styles).toMatch(
      /\.provider-card\[data-working\] \.familiar-slot::before\s*\{[^}]*opacity:[^}]*transform:\s*translate\(-50%, -50%\) scale/s,
    );
    expect(styles).not.toMatch(/\.provider-card\[data-working\] \.sprite\s*\{/);
    expect(styles).not.toMatch(/\.sprite\.(?:claude|codex)-mage\s*\{[^}]*drop-shadow\(0 0/s);
  });

  it("keeps provider art and ornament overflow visible", () => {
    expect(styles).toMatch(/#card section\s*\{[^}]*overflow:\s*visible/s);
    expect(styles).toMatch(
      /\.familiar-slot\s*\{[^}]*width:\s*70px[^}]*min-height:\s*64px[^}]*overflow:\s*visible/s,
    );
    expect(styles).toMatch(/\.sprite\s*\{[^}]*overflow:\s*visible/s);
  });

  it("adds animated XP sheen and circular gem glints with reduced-motion coverage", () => {
    expect(styles).toMatch(/#progress \.xpfill::before\s*\{[^}]*animation:/s);
    expect(styles).toMatch(
      /#progress \.xpfill::after\s*\{[^}]*radial-gradient\(circle[^}]*animation:/s,
    );
    const reducedMotion = styles.slice(
      styles.indexOf("@media (prefers-reduced-motion: reduce)"),
    );
    expect(reducedMotion).toMatch(
      /#progress \.xpfill::before,\s*#progress \.xpfill::after[^{]*\{[^}]*animation:\s*none/s,
    );
  });
});
