import type { AuraDescriptor } from "./aura-assets";

function loopPosition(elapsedMs: number, descriptor: AuraDescriptor): number {
  const elapsed = Number.isFinite(elapsedMs) ? elapsedMs : 0;
  const loopDuration = descriptor.frameHoldsMs.reduce(
    (sum, hold) => sum + hold,
    0,
  );
  const phaseAdjusted = elapsed + descriptor.phaseOffsetMs;
  return ((phaseAdjusted % loopDuration) + loopDuration) % loopDuration;
}

function frameAndDelay(
  elapsedMs: number,
  descriptor: AuraDescriptor,
): { frame: number; delay: number } {
  const position = loopPosition(elapsedMs, descriptor);
  let boundary = 0;
  for (let frame = 0; frame < descriptor.frameHoldsMs.length; frame += 1) {
    boundary += descriptor.frameHoldsMs[frame];
    if (position < boundary) {
      return { frame, delay: boundary - position };
    }
  }
  return { frame: 0, delay: descriptor.frameHoldsMs[0] };
}

export function auraFrameAt(
  elapsedMs: number,
  descriptor: AuraDescriptor,
  reducedMotion: boolean,
): number {
  return reducedMotion ? 0 : frameAndDelay(elapsedMs, descriptor).frame;
}

export function auraFrameDelayAt(
  elapsedMs: number,
  descriptor: AuraDescriptor,
  reducedMotion: boolean,
): number | undefined {
  return reducedMotion
    ? undefined
    : frameAndDelay(elapsedMs, descriptor).delay;
}
