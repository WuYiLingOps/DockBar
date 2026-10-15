import { assert } from "@std/assert/assert";
import * as DockBarLib from "./lib.ts";

Deno.test("Library is importable on background (no window)", () => {
  assert(!!DockBarLib);
});
