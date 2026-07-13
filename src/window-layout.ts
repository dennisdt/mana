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

export type LayoutRequest = { expanded: boolean; revision: number };

export function createLayoutIntent(initialExpanded: boolean) {
  let expanded = initialExpanded;
  let revision = 0;

  function isCurrent(request: LayoutRequest): boolean {
    return request.revision === revision && request.expanded === expanded;
  }

  return {
    get expanded(): boolean {
      return expanded;
    },
    request(nextExpanded: boolean): LayoutRequest {
      expanded = nextExpanded;
      revision += 1;
      return { expanded: nextExpanded, revision };
    },
    isCurrent,
    resetIfCurrent(request: LayoutRequest, nextExpanded: boolean): boolean {
      if (!isCurrent(request)) return false;
      expanded = nextExpanded;
      revision += 1;
      return true;
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
