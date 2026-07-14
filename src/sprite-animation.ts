export const SPRITE_FRAME_DURATION_MS = {
  idle: 287.5,
  working: 170,
  hover: 205,
} as const;

export const SPRITE_TICK_MS = 50;

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
