import { describe, expect, it, vi } from "vitest";
import tauriConfig from "../src-tauri/tauri.conf.json";
import {
  INITIAL_ROSTER_HEIGHT,
  ROSTER_WIDTH,
  WIDGET_ZOOM,
  createSerialQueue,
  rosterHeight,
  rosterOrigin,
  scaledRosterSize,
} from "./window-layout";

describe("permanent roster geometry", () => {
  it("uses the wider expanded roster from startup", () => {
    expect({ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }).toEqual({
      width: 456,
      height: 175,
    });
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );
    // The config window opens at the compact fixed zoom so first paint
    // matches what the frontend immediately resizes to.
    expect(mainWindow).toMatchObject({
      width: 456,
      height: 175,
    });
  });

  it("keeps the main webview rendering while the panel is inactive", () => {
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );

    expect(mainWindow).toMatchObject({
      backgroundThrottling: "disabled",
    });
  });

  it("keeps an origin that fits the active work area", () => {
    expect(
      rosterOrigin(
        { x: 500, y: 80 },
        { width: ROSTER_WIDTH, height: 175 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 500, y: 80 });
  });

  it("clamps both axes for a saved compact position near the edge", () => {
    expect(
      rosterOrigin(
        { x: 2300, y: 1700 },
        { width: ROSTER_WIDTH, height: 210 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 1968, y: 1380 });
  });

  it("reclamps an old 420px right-edge position for the wider roster", () => {
    expect(
      rosterOrigin(
        { x: 2040, y: 80 },
        { width: ROSTER_WIDTH, height: 210 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 1968, y: 80 });
  });
});

it("rounds measured card height plus the root border", () => {
  expect(rosterHeight(207.2)).toBe(210);
});

describe("fixed widget zoom", () => {
  it("declares a non-resizable window with no drag bounds", () => {
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );
    expect(mainWindow).toMatchObject({ resizable: false });
    expect(mainWindow).not.toHaveProperty("minWidth");
    expect(mainWindow).not.toHaveProperty("maxWidth");
  });

  it("renders at the compact fixed 1.0 zoom", () => {
    expect(WIDGET_ZOOM).toBe(1);
  });

  it("scales the roster size to whole logical pixels at the fixed zoom", () => {
    expect(scaledRosterSize(207.2, 1)).toEqual({ width: 456, height: 210 });
    expect(scaledRosterSize(207.2, WIDGET_ZOOM)).toEqual({ width: 456, height: 210 });
  });
});

it("serializes work and continues after a rejected operation", async () => {
  const events: string[] = [];
  const errors: unknown[] = [];
  const enqueue = createSerialQueue((error) => errors.push(error));
  let releaseFirst: () => void = () => undefined;
  const firstBlocked = new Promise<void>((resolve) => {
    releaseFirst = resolve;
  });
  const first = enqueue(async () => {
    events.push("first:start");
    await firstBlocked;
    events.push("first:end");
    throw new Error("window call failed");
  });
  const second = enqueue(async () => {
    events.push("second");
  });

  await vi.waitFor(() => expect(events).toEqual(["first:start"]));
  releaseFirst();
  await Promise.all([first, second]);

  expect(events).toEqual(["first:start", "first:end", "second"]);
  expect(errors).toHaveLength(1);
});
