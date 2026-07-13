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

export function createHoverIntent(
  setHovering: (value: boolean) => void,
  setExpanded: (value: boolean) => void,
) {
  let collapseTimer: ReturnType<typeof setTimeout> | undefined;
  return {
    enter(): void {
      clearTimeout(collapseTimer);
      collapseTimer = undefined;
      setHovering(true);
      setExpanded(true);
    },
    leave(): void {
      setHovering(false);
      clearTimeout(collapseTimer);
      collapseTimer = setTimeout(() => setExpanded(false), COLLAPSE_DELAY_MS);
    },
  };
}
