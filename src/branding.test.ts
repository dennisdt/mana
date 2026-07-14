// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const root = new URL("../", import.meta.url);
const packageJson = JSON.parse(readFileSync(new URL("package.json", root), "utf8"));
const packageLock = JSON.parse(readFileSync(new URL("package-lock.json", root), "utf8"));
const tauriConfig = JSON.parse(
  readFileSync(new URL("src-tauri/tauri.conf.json", root), "utf8"),
);
const cargoToml = readFileSync(new URL("src-tauri/Cargo.toml", root), "utf8");
const cargoLock = readFileSync(new URL("src-tauri/Cargo.lock", root), "utf8");
const html = readFileSync(new URL("index.html", root), "utf8");
const readme = readFileSync(new URL("README.md", root), "utf8");
const iconUrl = new URL("src-tauri/icons/mana-potion-master.png", root);

describe("Mana product identity", () => {
  it("uses the visible Mana name while preserving internal identifiers", () => {
    expect(packageJson.name).toBe("mana");
    expect(tauriConfig.productName).toBe("Mana");
    expect(tauriConfig.identifier).toBe("com.vantasoft.mana");
    expect(tauriConfig.app.windows[0].title).toBe("Mana");
    expect(cargoToml).toMatch(/^name = "mana"$/m);
    expect(html).toContain("<title>Mana</title>");
    expect(readme).toMatch(/^# Mana$/m);
    expect(readme).toContain("Both are read-only: Mana re-reads credentials");
    expect(readme).toContain("bundle/macos/Mana.app /Applications/");
  });

  it("keeps release version 0.4.4 synchronized", () => {
    expect(packageJson.version).toBe("0.4.4");
    expect(packageLock.version).toBe("0.4.4");
    expect(packageLock.packages[""].version).toBe("0.4.4");
    expect(tauriConfig.version).toBe("0.4.4");
    expect(cargoToml).toMatch(/^version = "0\.4\.4"$/m);
    expect(cargoLock).toMatch(/name = "mana"\nversion = "0\.4\.4"/);
  });

  it("preserves the approved square icon master", () => {
    expect(existsSync(iconUrl)).toBe(true);
    const png = readFileSync(iconUrl);
    expect(png.subarray(1, 4).toString("ascii")).toBe("PNG");
    expect(png.readUInt32BE(16)).toBe(1254);
    expect(png.readUInt32BE(20)).toBe(1254);
  });
});
