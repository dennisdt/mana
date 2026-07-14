export const METER_WIDTH = 144;
export const METER_HEIGHT = 20;
export const METER_INSET_X = 14;
export const METER_INSET_Y = 6;
export const METER_CHANNEL_WIDTH = METER_WIDTH - METER_INSET_X * 2;
export const METER_CHANNEL_HEIGHT = METER_HEIGHT - METER_INSET_Y * 2;

export function meterFillPixels(percent: number): number {
  const clamped = Math.max(0, Math.min(100, percent));
  return Math.round((clamped / 100) * METER_CHANNEL_WIDTH);
}
