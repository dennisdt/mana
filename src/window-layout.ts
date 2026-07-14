export const ROSTER_WIDTH = 440;
export const INITIAL_ROSTER_HEIGHT = 175;

type Point = { x: number; y: number };
type Size = { width: number; height: number };
type Rect = Point & Size;

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
