import { test } from "node:test";
import assert from "node:assert/strict";
import { createSwipe } from "./swipe.ts";

function run(events: [number, number][], start = 0, step = 16) {
  const calls: number[] = [];
  const onWheel = createSwipe((d) => calls.push(d));
  let t = start;
  for (const [deltaX, deltaY] of events) onWheel({ deltaX, deltaY, timeStamp: (t += step) });
  return { calls, onWheel, t };
}

const repeat = (n: number, e: [number, number]) => Array.from({ length: n }, () => e);

test("one long gesture with momentum fires once", () => {
  assert.deepEqual(run(repeat(40, [10, 0])).calls, [1]);
});

test("new gesture after quiet gap fires again", () => {
  const { calls, onWheel, t } = run(repeat(10, [10, 0]));
  let ts = t + 300;
  for (let i = 0; i < 10; i++) onWheel({ deltaX: -10, deltaY: 0, timeStamp: (ts += 16) });
  assert.deepEqual(calls, [1, -1]);
});

test("vertical scroll is ignored", () => {
  assert.deepEqual(run(repeat(20, [5, 20])).calls, []);
});

test("below threshold does nothing", () => {
  assert.deepEqual(run(repeat(3, [10, 0])).calls, []);
});
