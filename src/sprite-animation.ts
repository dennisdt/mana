export const SPRITE_FRAME_DURATION_MS = {
  idle: 575,
  working: 340,
  hover: 410,
} as const;

function frameDuration(state: string | undefined): number {
  if (state === "working") return SPRITE_FRAME_DURATION_MS.working;
  if (state === "hover") return SPRITE_FRAME_DURATION_MS.hover;
  return SPRITE_FRAME_DURATION_MS.idle;
}

export function spriteFrameAt(
  elapsedMs: number,
  state: string | undefined,
  reducedMotion: boolean,
): number {
  if (reducedMotion) return 0;
  const elapsed = Number.isFinite(elapsedMs) ? Math.max(0, elapsedMs) : 0;
  return Math.floor(elapsed / frameDuration(state)) % 4;
}

export function spriteFrameDelayAt(
  elapsedMs: number,
  state: string | undefined,
  reducedMotion: boolean,
): number | undefined {
  if (reducedMotion) return undefined;
  const elapsed = Number.isFinite(elapsedMs) ? Math.max(0, elapsedMs) : 0;
  const duration = frameDuration(state);
  return duration - (elapsed % duration);
}
