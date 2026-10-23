/** New index for a navigation key, clamped to the list (no wrap); null if the key is not ours. */
export function navigate(index: number, length: number, key: string): number | null {
  if (length === 0) return null;
  const last = length - 1;
  switch (key) {
    case "ArrowLeft":
      return Math.max(index - 1, 0);
    case "ArrowRight":
      return Math.min(index + 1, last);
    case "Home":
      return 0;
    case "End":
      return last;
    default:
      return null;
  }
}
