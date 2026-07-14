# Mana Menu-Bar Template Icon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the full-color square tray thumbnail with the approved native macOS filled-potion template glyph while preserving the full-color application icon.

**Architecture:** Store a compact SVG as the reproducible tray source and a 36x36 PNG as the embedded runtime asset. Decode the PNG through Tauri's existing `image-png` support, mark it as a macOS template image, and keep application icon generation and tray behavior independent.

**Tech Stack:** Rust, Tauri 2, SVG, PNG, Vitest, Cargo tests, macOS CuaDriver

## Global Constraints

- Use selected direction A: a centered filled-potion silhouette with a transparent liquid cutout.
- Runtime tray PNG is exactly 36x36 RGBA on a transparent canvas.
- Use `.icon_as_template(true)` so macOS owns the foreground color.
- Keep `src-tauri/icons/mana-potion-master.png` and every full-color application icon unchanged.
- Keep `ActivationPolicy::Accessory`; the running Mana widget must never appear in the Dock.
- Visible name remains `Mana`; npm package and Rust crate names remain `mana`; bundle identifier remains `com.vantasoft.mana`.
- Release version is exactly `0.4.5` in npm, Cargo, and Tauri metadata.
- Keep widget layout, provider artwork, mana bars, motion, polling, activity detection, panel positioning, and tray interaction behavior unchanged.

---

### Task 1: Dedicated Tray Template Asset and Runtime Integration

**Files:**
- Create: `src-tauri/icons/tray-template.svg`
- Create: `src-tauri/icons/tray-template.png`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/branding.test.ts`

**Interfaces:**
- Produces: `fn tray_template_icon() -> tauri::Result<tauri::image::Image<'static>>`.
- Consumes: `include_bytes!("../icons/tray-template.png")` and Tauri's existing `image-png` feature.
- Produces: `TrayIconBuilder` configured with `.icon(tray_template_icon()?)`, `.icon_as_template(true)`, and `.tooltip("Mana")`.

- [ ] **Step 1: Add failing runtime-source assertions**

Extend `src/branding.test.ts` to require `tray_template_icon()`, `include_bytes!("../icons/tray-template.png")`, `.icon_as_template(true)`, `.tooltip("Mana")`, `Quit Mana`, and `ActivationPolicy::Accessory`; also require that `app.default_window_icon()` is absent from tray construction.

- [ ] **Step 2: Add the failing Rust asset test**

Add a `#[cfg(test)]` module in `src-tauri/src/lib.rs` that calls `tray_template_icon()` and asserts width and height are 36, all four corner alpha values are zero, opaque-pixel coverage is non-zero but below 70 percent, and the alpha-weighted bounds center lies within two pixels of the canvas center.

- [ ] **Step 3: Run focused tests and confirm failure**

Run: `npm test -- --run src/branding.test.ts && cargo test --manifest-path src-tauri/Cargo.toml tray_template`

Expected: frontend assertions fail because the dedicated asset/runtime API is absent; the Rust test cannot pass until the asset and helper exist.

- [ ] **Step 4: Create the approved vector source and raster**

Create a 36-unit square SVG with a transparent canvas and one black filled-potion path. Use an alpha cutout for the liquid line, retain at least three pixels of exterior padding at 36px, and omit background, shadow, glow, color, clouds, text, and sparkle decoration.

Render deterministically on macOS:

```bash
sips -s format png -z 36 36 src-tauri/icons/tray-template.svg --out src-tauri/icons/tray-template.png
```

Verify: `sips -g pixelWidth -g pixelHeight -g hasAlpha src-tauri/icons/tray-template.png` reports 36, 36, and yes.

- [ ] **Step 5: Integrate the template icon**

Decode `include_bytes!("../icons/tray-template.png")` with `tauri::image::Image::from_bytes`, call `.to_owned()`, and use that helper in `TrayIconBuilder`. Add `.icon_as_template(true)`, `.tooltip("Mana")`, and correct the quit label to `Quit Mana` without changing menu callbacks.

- [ ] **Step 6: Run focused and complete tests**

Run: `npm test -- --run src/branding.test.ts && cargo test --manifest-path src-tauri/Cargo.toml tray_template && npm test && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: focused and complete frontend/Rust suites PASS.

- [ ] **Step 7: Commit**

```bash
git add src/branding.test.ts src-tauri/src/lib.rs src-tauri/icons/tray-template.svg src-tauri/icons/tray-template.png
git commit -m "fix: add native Mana menu-bar icon"
```

### Task 2: Versioned Native Release and Visual QA

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: `/Applications/Mana.app` version `0.4.5` with bundle identifier `com.vantasoft.mana`.
- Consumes: the template tray asset and runtime integration from Task 1.

- [ ] **Step 1: Update failing version assertions**

Change `src/branding.test.ts` version expectations from `0.4.4` to `0.4.5` for npm, npm lock, Cargo, Cargo lock, and Tauri metadata.

- [ ] **Step 2: Run the branding test and confirm failure**

Run: `npm test -- --run src/branding.test.ts`

Expected: FAIL because release metadata still reports `0.4.4`.

- [ ] **Step 3: Synchronize version metadata**

Set version `0.4.5` in `package.json`, both root entries in `package-lock.json`, `src-tauri/Cargo.toml`, the `mana` package entry in `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`. Do not change names or the bundle identifier.

- [ ] **Step 4: Run the full production verification matrix**

Run: `npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml && npm run tauri build`

Expected: 66 or more frontend tests pass, 19 or more Rust tests pass, and `src-tauri/target/release/bundle/macos/Mana.app` builds successfully.

- [ ] **Step 5: Install and verify metadata and hashes**

Back up the current `/Applications/Mana.app` to a timestamped `/tmp` path, install the new bundle, and confirm `Info.plist` reports `Mana`, `com.vantasoft.mana`, and `0.4.5`. Compare SHA-256 hashes for the built and installed executables, `icon.icns`, and embedded `tray-template.png`.

- [ ] **Step 6: Relaunch and visually inspect with CuaDriver**

Request permission before quitting the running app, then use CuaDriver to quit and relaunch `com.vantasoft.mana`. Capture the real menu bar and verify the glyph has no colored square, matches neighboring item height, reads as a potion, adapts to the current menu-bar color, and is not clipped or crowded. Confirm Mana is absent from the Dock while its accessory process and menu-bar item remain running.

- [ ] **Step 7: Commit**

```bash
git add package.json package-lock.json src/branding.test.ts src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "chore: release Mana 0.4.5"
```

### Task 3: Final Integrated Review

**Files:**
- Review: every change since the implementation-plan baseline

**Interfaces:**
- Consumes: completed Tasks 1 and 2.
- Produces: evidence that the installed release satisfies the complete approved menu-bar specification.

- [ ] **Step 1: Review the complete diff**

Check for accidental changes to full-color icons, tray callbacks, accessory activation policy, window behavior, package/crate names, or bundle identifier. Confirm the SVG and PNG represent the same filled-potion silhouette, the template flag is macOS-specific behavior provided by Tauri, and no Dock icon is introduced.

- [ ] **Step 2: Run final verification**

Run: `npm test && npm run build && cargo test --manifest-path src-tauri/Cargo.toml`

Expected: every test and build exits zero.

- [ ] **Step 3: Confirm repository and installed state**

Run: `git status --short && git log --oneline -6`

Expected: only the pre-existing unrelated `.DS_Store` remains untracked, implementation commits are present, and `/Applications/Mana.app` is the verified `0.4.5` bundle.
