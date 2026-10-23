export type WheelLike = { deltaX: number; deltaY: number; timeStamp: number };

/**
 * Two-finger horizontal swipe -> one step. After firing, stays locked until wheel events
 * (including macOS momentum) go quiet for `quietMs`, so one gesture = one frame.
 */
export function createSwipe(
  onSwipe: (dir: 1 | -1) => void,
  threshold = 60,
  quietMs = 200,
): (e: WheelLike) => void {
  let acc = 0;
  let last = -Infinity;
  let locked = false;
  return ({ deltaX, deltaY, timeStamp }) => {
    if (Math.abs(deltaY) > Math.abs(deltaX)) return;
    if (timeStamp - last > quietMs) {
      acc = 0;
      locked = false;
    }
    last = timeStamp;
    if (locked) return;
    acc += deltaX;
    if (Math.abs(acc) >= threshold) {
      locked = true;
      onSwipe(acc > 0 ? 1 : -1);
    }
  };
}
