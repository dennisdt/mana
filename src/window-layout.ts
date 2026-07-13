export const COLLAPSED_WIDTH = 340;
export const COLLAPSED_HEIGHT = 48;
export const EXPANDED_WIDTH = 420;
export const COLLAPSE_DELAY_MS = 150;

type Point = { x: number; y: number };
type Rect = Point & { width: number; height: number };

export function expandedOrigin(origin: Point, workArea: Rect, scaleFactor: number): Point {
  const maximumX = workArea.x + workArea.width - EXPANDED_WIDTH * scaleFactor;
  return { x: Math.max(workArea.x, Math.min(origin.x, maximumX)), y: origin.y };
}

export function expandedHeight(cardScrollHeight: number): number {
  return Math.ceil(cardScrollHeight + 2);
}

export function collapsedOriginFromExpanded(
  position: Point,
  expandedOffsetX: number,
): Point {
  return { x: position.x - expandedOffsetX, y: position.y };
}

export function createSerialQueue(onError: (error: unknown) => void) {
  let tail: Promise<void> = Promise.resolve();
  return (task: () => Promise<void>): Promise<void> => {
    tail = tail.then(task).catch((error) => {
      onError(error);
    });
    return tail;
  };
}

export function createRequestRevision() {
  let current = 0;
  return {
    issue(): number {
      current += 1;
      return current;
    },
    isCurrent(revision: number): boolean {
      return revision === current;
    },
  };
}

export function createHoverIntent(
  setHovering: (value: boolean) => void,
  setExpanded: (value: boolean) => void,
  isMoving: () => boolean = () => false,
) {
  let collapseTimer: ReturnType<typeof setTimeout> | undefined;
  let collapsePending = false;

  function collapseWhenSettled(): void {
    collapseTimer = undefined;
    if (isMoving()) {
      collapsePending = true;
      return;
    }
    collapsePending = false;
    setExpanded(false);
  }

  return {
    enter(): void {
      clearTimeout(collapseTimer);
      collapseTimer = undefined;
      collapsePending = false;
      setHovering(true);
      setExpanded(true);
    },
    leave(): void {
      setHovering(false);
      clearTimeout(collapseTimer);
      collapsePending = false;
      collapseTimer = setTimeout(collapseWhenSettled, COLLAPSE_DELAY_MS);
    },
    movementSettled(): void {
      if (!collapsePending || isMoving()) return;
      collapseWhenSettled();
    },
  };
}
