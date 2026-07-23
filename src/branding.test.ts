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
const tauriSource = readFileSync(new URL("src-tauri/src/lib.rs", root), "utf8");
const progressStoreSource = readFileSync(new URL("src-tauri/src/progress_store.rs", root), "utf8");
const iconUrl = new URL("src-tauri/icons/mana-potion-master.png", root);
const infoPlistUrl = new URL("src-tauri/Info.plist", root);

function extractBalancedRustBody(
  source: string,
  bodyStart: number,
  description: string,
): string {
  if (bodyStart === -1) {
    throw new Error(`Rust body not found: ${description}`);
  }

  let depth = 0;
  for (let index = bodyStart; index < source.length; index += 1) {
    if (source[index] === "{") {
      depth += 1;
    } else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(bodyStart + 1, index);
      }
    }
  }

  throw new Error(`Rust body is unbalanced: ${description}`);
}

function extractRustMethodBody(
  source: string,
  typeName: string,
  functionName: string,
): string {
  const implPattern = new RegExp(`\\bimpl\\s+${typeName}\\s*\\{`);
  const implMatch = implPattern.exec(source);
  if (!implMatch) {
    throw new Error(`Rust implementation not found: ${typeName}`);
  }

  const implBody = extractBalancedRustBody(
    source,
    source.indexOf("{", implMatch.index),
    `impl ${typeName}`,
  );
  const functionPattern = new RegExp(`\\bfn\\s+${functionName}\\s*\\(`);
  const functionMatch = functionPattern.exec(implBody);
  if (!functionMatch) {
    throw new Error(`Rust method not found: ${typeName}::${functionName}`);
  }

  return extractBalancedRustBody(
    implBody,
    implBody.indexOf("{", functionMatch.index),
    `${typeName}::${functionName}`,
  );
}

describe("Mana product identity", () => {
  it("uses the visible Mana name while preserving internal identifiers", () => {
    expect(packageJson.name).toBe("mana");
    expect(tauriConfig.productName).toBe("Mana");
    expect(tauriConfig.identifier).toBe("com.vantasoft.mana");
    expect(progressStoreSource).toContain(
      'pub const PRIMARY_PROGRESS_FILENAME: &str = "progress.json";',
    );
    expect(progressStoreSource).toContain(
      'pub const BACKUP_PROGRESS_FILENAME: &str = "progress.json.bak";',
    );
    expect(extractRustMethodBody(progressStoreSource, "ProgressStore", "load")).toMatch(
      /app_data_dir\(\)[\s\S]*?\.join\(PRIMARY_PROGRESS_FILENAME\)/,
    );
    expect(progressStoreSource).toContain("dir.join(BACKUP_PROGRESS_FILENAME)");
    expect(tauriConfig.app.windows[0].title).toBe("Mana");
    expect(cargoToml).toMatch(/^name = "mana"$/m);
    expect(html).toContain("<title>Mana</title>");
    expect(readme).toMatch(/^# Mana$/m);
    expect(readme).toContain("Credential access is read-only.");
    expect(readme).toContain(
      "Mana re-reads credentials for each poll and never writes or refreshes them.",
    );
    expect(readme).toContain("src-tauri/target/release/bundle/macos/Mana.app");
  });

  it("keeps release version 0.4.5 synchronized", () => {
    expect(packageJson.version).toBe("0.4.5");
    expect(packageLock.version).toBe("0.4.5");
    expect(packageLock.packages[""].version).toBe("0.4.5");
    expect(tauriConfig.version).toBe("0.4.5");
    expect(cargoToml).toMatch(/^version = "0\.4\.5"$/m);
    expect(cargoLock).toMatch(/name = "mana"\nversion = "0\.4\.5"/);
  });

  it("preserves the approved square icon master", () => {
    expect(existsSync(iconUrl)).toBe(true);
    const png = readFileSync(iconUrl);
    expect(png.subarray(1, 4).toString("ascii")).toBe("PNG");
    expect(png.readUInt32BE(16)).toBe(1254);
    expect(png.readUInt32BE(20)).toBe(1254);
  });

  it("uses a dedicated native template icon for the menu bar", () => {
    expect(tauriSource).toContain("fn tray_template_icon()");
    expect(tauriSource).toContain('include_bytes!("../icons/tray-template.png")');
    expect(tauriSource).toContain(".icon(tray_template_icon()?)");
    expect(tauriSource).toContain(".icon_as_template(true)");
    expect(tauriSource).toContain('.tooltip("Mana")');
    expect(tauriSource).toContain('"Quit Mana"');
    expect(tauriSource).toContain("ActivationPolicy::Accessory");
    expect(tauriSource).not.toContain("app.default_window_icon()");
  });

  it("declares Mana as a macOS agent application", () => {
    expect(existsSync(infoPlistUrl)).toBe(true);
    const infoPlist = readFileSync(infoPlistUrl, "utf8");
    expect(infoPlist).toMatch(/<key>LSUIElement<\/key>\s*<true\s*\/>/);
  });
});
