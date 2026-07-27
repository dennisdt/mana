// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const styles = readFileSync(new URL("./styles.css", import.meta.url), "utf8");
const mainSource = readFileSync(new URL("./main.ts", import.meta.url), "utf8");
const librs = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const indexHtml = readFileSync(new URL("../index.html", import.meta.url), "utf8");

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
    expect(styles).toContain(".aura");
    expect(styles).not.toContain(".familiar-slot::before");
    expect(styles).not.toContain(".familiar-slot::after");
    expect(styles).toMatch(
      /#card section\s*\{[^}]*grid-template-columns:\s*70px minmax\(0, 1fr\)[^}]*gap:\s*24px/s,
    );
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
    expect(artRule).toMatch(
      /top:\s*calc\(50% \+ var\(--sprite-y-offset,\s*0px\)\)/,
    );
    expect(artRule).toMatch(
      /left:\s*calc\(50% \+ var\(--sprite-x-offset,\s*0px\)\)/,
    );
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

  it("optically grounds both provider sprites without moving the provider layout", () => {
    expect(styles).toMatch(
      /\.sprite::before\s*\{[^}]*top:\s*calc\(50% \+ var\(--sprite-y-offset,\s*0px\)\)/s,
    );
    expect(styles).toMatch(
      /\.sprite::before\s*\{[^}]*left:\s*calc\(50% \+ var\(--sprite-x-offset,\s*0px\)\)/s,
    );
    expect(styles).toMatch(
      /\.sprite\.claude-mage\s*\{[^}]*--sprite-x-offset:\s*-4px[^}]*--sprite-y-offset:\s*10px/s,
    );
    expect(styles).toMatch(
      /\.sprite\.codex-mage\s*\{[^}]*--sprite-x-offset:\s*-3px[^}]*--sprite-y-offset:\s*8px/s,
    );
    expect(styles).toMatch(
      /\.familiar-slot\s*\{[^}]*align-items:\s*center[^}]*justify-content:\s*center/s,
    );
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
    expect(styles).toMatch(/#glass\s*\{[^}]*flex-direction:\s*column/s);
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

  it("keeps one continuous fallback outline on the flush glass", () => {
    expect(styles).toMatch(
      /#glass\s*\{[^}]*inset:\s*0[^}]*border:\s*1px solid[^}]*border-radius:\s*var\(--hud-radius\)/s,
    );
    expect(styles).not.toMatch(/border-style:\s*(?:dotted|dashed)/);
    expect(styles).not.toContain("--corner-tick");
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

describe("generated application perimeter", () => {
  it("uses a transparent shell and one flush glass surface", () => {
    const root = styles.match(/#root\s*\{([^}]*)\}/s)?.[1] ?? "";
    const glass = styles.match(/#glass\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(root).toMatch(/background:\s*transparent/);
    expect(root).not.toMatch(/\bborder\s*:/);
    expect(root).not.toMatch(/\bbox-shadow\s*:/);
    expect(glass).toMatch(/position:\s*absolute/);
    expect(glass).toMatch(/inset:\s*0/);
    expect(glass).toMatch(/width:\s*100%/);
    expect(glass).toMatch(/height:\s*100%/);
    expect(styles).not.toMatch(/#root::(?:before|after)/);
  });

  it("composes native pixel-art pieces without stretching complete frames", () => {
    const perimeterStyles = styles.slice(styles.indexOf("#perimeter"));
    expect(styles).toMatch(
      /#perimeter\s*\{[^}]*position:\s*absolute[^}]*inset:\s*0[^}]*pointer-events:\s*none/s,
    );
    expect(styles).toMatch(
      /\.frame-corner\s*\{[^}]*width:\s*32px[^}]*height:\s*32px[^}]*image-rendering:\s*pixelated/s,
    );
    expect(styles).toMatch(
      /\.frame-crest\s*\{[^}]*display:\s*none/s,
    );
    const horizontal =
      styles.match(
        /\.frame-rail--top,\s*\.frame-rail--bottom\s*\{([^}]*)\}/s,
      )?.[1] ?? "";
    const vertical =
      styles.match(
        /\.frame-rail--right,\s*\.frame-rail--left\s*\{([^}]*)\}/s,
      )?.[1] ?? "";
    expect(horizontal).toMatch(/background-size:\s*64px 16px/);
    expect(horizontal).toMatch(/background-repeat:\s*repeat-x/);
    expect(vertical).toMatch(/background-size:\s*16px 64px/);
    expect(vertical).toMatch(/background-repeat:\s*repeat-y/);
    expect(perimeterStyles).toContain("background-size: 64px 16px");
  });

  it("underlaps rails beneath transparent corner padding", () => {
    const horizontal =
      styles.match(
        /\.frame-rail--top,\s*\.frame-rail--bottom\s*\{([^}]*)\}/s,
      )?.[1] ?? "";
    const vertical =
      styles.match(
        /\.frame-rail--right,\s*\.frame-rail--left\s*\{([^}]*)\}/s,
      )?.[1] ?? "";
    const corner = styles.match(/\.frame-corner\s*\{([^}]*)\}/s)?.[1] ?? "";

    expect(horizontal).toMatch(/right:\s*16px/);
    expect(horizontal).toMatch(/left:\s*16px/);
    expect(vertical).toMatch(/top:\s*16px/);
    expect(vertical).toMatch(/bottom:\s*16px/);
    expect(corner).toMatch(/z-index:\s*3/);
  });

  it("evenly spaces one ornament lane per side", () => {
    expect(styles).toMatch(
      /\.frame-ornaments--top,[^}]*\.frame-ornaments--bottom\s*\{[^}]*display:\s*grid[^}]*grid-auto-flow:\s*column[^}]*justify-content:\s*space-evenly/s,
    );
    expect(styles).toMatch(
      /\.frame-ornaments--right,[^}]*\.frame-ornaments--left\s*\{[^}]*display:\s*grid[^}]*grid-auto-flow:\s*row[^}]*align-content:\s*space-evenly/s,
    );
  });

  it("limits motion to prestige seven through ten highlights and flashes", () => {
    expect(styles).toMatch(
      /#perimeter\[data-prestige="7"\][^{,]*\.frame-rail::after/s,
    );
    expect(styles).toContain("@keyframes prestige-rail-light");
    expect(styles).toContain("@keyframes prestige-corner-flash");
    expect(styles).not.toContain("champion-radiance");
    expect(styles).not.toContain("godlike-halo");
    const reducedMotion = styles.slice(
      styles.indexOf("@media (prefers-reduced-motion: reduce)"),
    );
    expect(reducedMotion).toMatch(
      /\.frame-rail::after,[^}]*\.frame-corner::after\s*\{[^}]*animation:\s*none/s,
    );
  });

  it("retires the old CSS frame and stacked badge paths", () => {
    expect(indexHtml).toContain('<div id="glass">');
    expect(indexHtml).toContain('<div id="perimeter" aria-hidden="true"></div>');
    expect(indexHtml).not.toContain('id="frame"');
    expect(indexHtml).not.toContain('class="badges"');
    expect(styles).not.toContain("#frame");
    expect(styles).not.toContain(".badge");
    expect(mainSource).not.toContain("badgeSlots");
    expect(mainSource).not.toContain("badgeHtml");
    expect(mainSource).not.toContain("/badges/");
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

  it("renders fixed authored aura cells behind grounded sprites without a generic glow", () => {
    expect(styles).not.toContain("activity-signal");
    expect(styles).not.toContain(".familiar-slot::before");
    expect(styles).not.toMatch(/\.familiar-slot[^}]*radial-gradient/s);
    const aura = styles.match(/\.aura\s*\{([^}]*)\}/s)?.[1] ?? "";
    expect(aura).toMatch(/position:\s*absolute/);
    expect(aura).toMatch(/z-index:\s*1/);
    expect(aura).toMatch(/top:\s*calc\(50% - 12px\)/);
    expect(aura).toMatch(/left:\s*50%/);
    expect(aura).toMatch(/width:\s*96px/);
    expect(aura).toMatch(/height:\s*96px/);
    expect(aura).toMatch(/background-image:\s*var\(--aura-atlas,\s*none\)/);
    expect(aura).toMatch(
      /background-size:\s*calc\(var\(--aura-frame-count,\s*1\) \* 96px\) 96px/,
    );
    expect(aura).toMatch(/image-rendering:\s*pixelated/);
    expect(aura).toMatch(/pointer-events:\s*none/);
    expect(aura).toMatch(/transform:\s*translate\(-50%,\s*-50%\)/);
    expect(aura).toMatch(/zoom:\s*var\(--sprite-resolution-scale,\s*1\)/);
    expect(styles).toMatch(/\.sprite\s*\{[^}]*z-index:\s*2/s);
    for (let frame = 0; frame < 8; frame += 1) {
      const xPosition = frame === 0 ? "0" : `-${frame * 96}px`;
      expect(styles).toMatch(
        new RegExp(
          `\\.aura\\[data-frame="${frame}"\\]\\s*\\{[^}]*background-position-x:\\s*${xPosition}`,
          "s",
        ),
      );
    }
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
      /\.aura\[data-frame\]\s*\{[^}]*background-position-x:\s*0/s,
    );
    expect(reducedMotion).toMatch(
      /#progress \.xpfill::before,\s*#progress \.xpfill::after[^{]*\{[^}]*animation:\s*none/s,
    );
  });
});
