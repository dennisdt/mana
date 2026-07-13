export const METER_WIDTH = 128;
export const METER_HEIGHT = 16;
export const METER_INSET_X = 3;
export const METER_INNER_WIDTH = METER_WIDTH - METER_INSET_X * 2;

export function meterFillPixels(percent: number): number {
  const clamped = Math.max(0, Math.min(100, percent));
  return Math.round((clamped / 100) * METER_INNER_WIDTH);
}
