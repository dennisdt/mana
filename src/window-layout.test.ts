import { describe, expect, it, vi } from "vitest";
import tauriConfig from "../src-tauri/tauri.conf.json";
import {
  INITIAL_ROSTER_HEIGHT,
  ROSTER_WIDTH,
  createSerialQueue,
  rosterHeight,
  rosterOrigin,
} from "./window-layout";

describe("permanent roster geometry", () => {
  it("uses the wider expanded roster from startup", () => {
    expect({ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }).toEqual({
      width: 440,
      height: 175,
    });
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );
    expect(mainWindow).toMatchObject({
      width: ROSTER_WIDTH,
      height: INITIAL_ROSTER_HEIGHT,
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
    ).toEqual({ x: 2000, y: 1380 });
  });

  it("reclamps an old 420px right-edge position for the wider roster", () => {
    expect(
      rosterOrigin(
        { x: 2040, y: 80 },
        { width: ROSTER_WIDTH, height: 210 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 2000, y: 80 });
  });
});

it("rounds measured card height plus the root border", () => {
  expect(rosterHeight(207.2)).toBe(210);
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
