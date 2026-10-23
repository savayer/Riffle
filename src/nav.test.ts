import { test } from "node:test";
import assert from "node:assert/strict";
import { navigate } from "./nav.ts";

test("edges do not wrap", () => {
  assert.equal(navigate(0, 5, "ArrowLeft"), 0);
  assert.equal(navigate(4, 5, "ArrowRight"), 4);
  assert.equal(navigate(2, 5, "ArrowRight"), 3);
  assert.equal(navigate(2, 5, "ArrowLeft"), 1);
  assert.equal(navigate(2, 5, "Home"), 0);
  assert.equal(navigate(2, 5, "End"), 4);
});

test("other keys and empty list return null", () => {
  assert.equal(navigate(2, 5, "a"), null);
  assert.equal(navigate(0, 0, "ArrowRight"), null);
});
