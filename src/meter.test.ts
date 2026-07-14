import { describe, expect, it } from "vitest";
import {
  METER_CHANNEL_HEIGHT,
  METER_HEIGHT,
  METER_CHANNEL_WIDTH,
  METER_INSET_X,
  METER_INSET_Y,
  METER_WIDTH,
  meterFillPixels,
} from "./meter";

describe("fantasy mana meter", () => {
  it("uses the approved frame and live-core dimensions", () => {
    expect({
      width: METER_WIDTH,
      height: METER_HEIGHT,
      insetX: METER_INSET_X,
      insetY: METER_INSET_Y,
      channelWidth: METER_CHANNEL_WIDTH,
      channelHeight: METER_CHANNEL_HEIGHT,
    }).toEqual({
      width: 144,
      height: 20,
      insetX: 14,
      insetY: 6,
      channelWidth: 116,
      channelHeight: 8,
    });
  });

  it.each([
    [-1, 0],
    [0, 0],
    [1, 1],
    [29, 34],
    [30, 35],
    [50, 58],
    [55, 64],
    [99, 115],
    [100, 116],
    [101, 116],
  ])("maps %s percent to %s core pixels", (percent, pixels) => {
    expect(meterFillPixels(percent)).toBe(pixels);
  });
});
