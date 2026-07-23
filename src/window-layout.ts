export const ROSTER_WIDTH = 456;
export const INITIAL_ROSTER_HEIGHT = 175;
export const FRAME_BLEED = 24;
export const WINDOW_WIDTH = ROSTER_WIDTH + FRAME_BLEED * 2;

/// The HUD renders at its compact authored scale. The window itself is
/// deliberately not user-resizable.
export const WIDGET_ZOOM = 1;

type Point = { x: number; y: number };
type Size = { width: number; height: number };
type Rect = Point & Size;

export function scaledRosterSize(cardScrollHeight: number, scale: number): Size {
  return {
    width: Math.round(WINDOW_WIDTH * scale),
    height: Math.ceil(
      (rosterHeight(cardScrollHeight) + FRAME_BLEED * 2) * scale,
    ),
  };
}

export function rosterOrigin(
  origin: Point,
  size: Size,
  workArea: Rect,
  scaleFactor: number,
): Point {
  const maximumX = workArea.x + workArea.width - size.width * scaleFactor;
  const maximumY = workArea.y + workArea.height - size.height * scaleFactor;
  return {
    x: Math.max(workArea.x, Math.min(origin.x, maximumX)),
    y: Math.max(workArea.y, Math.min(origin.y, maximumY)),
  };
}

export function rosterHeight(cardScrollHeight: number): number {
  return Math.ceil(cardScrollHeight + 2);
}

export function createSerialQueue(onError: (error: unknown) => void) {
  let tail: Promise<void> = Promise.resolve();
  return (task: () => Promise<void>): Promise<void> => {
    tail = tail.then(task).catch((error) => onError(error));
    return tail;
  };
}
