import { describe, expect, it } from "vitest";
import {
  METER_HEIGHT,
  METER_INNER_WIDTH,
  METER_WIDTH,
  meterFillPixels,
} from "./meter";

describe("fixed pixel meter", () => {
  it("uses the approved frame and interior dimensions", () => {
    expect({ width: METER_WIDTH, height: METER_HEIGHT, inner: METER_INNER_WIDTH })
      .toEqual({ width: 128, height: 16, inner: 122 });
  });

  it.each([
    [-1, 0],
    [0, 0],
    [1, 1],
    [29, 35],
    [30, 37],
    [55, 67],
    [99, 121],
    [100, 122],
    [101, 122],
  ])("maps %s percent to %s interior pixels", (percent, pixels) => {
    expect(meterFillPixels(percent)).toBe(pixels);
  });
});
