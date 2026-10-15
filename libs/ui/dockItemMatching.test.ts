import { isSameApp } from "./dockItemMatching.ts";
import assert from "node:assert/strict";

// tsc has no Deno global types (they live in the deno toolchain), so declare
// the minimum surface the test uses; at runtime the real global is picked up.
declare const Deno: { test: (name: string, fn: () => void) => void };

function win(umid: string | null, path: string) {
  return { umid, path };
}

function item(umid: string | null, path: string, relaunchCommand: string | null = null) {
  return { umid, path, relaunchCommand };
}

Deno.test("same umid matches even with different paths", () => {
  assert.ok(isSameApp(item("App\\id", "C:\\lnk\\app.lnk"), win("App\\id", "C:\\real\\target.exe")));
});

Deno.test("different umids do not block an exe-path match", () => {
  // regression: shortcut umid differing from the window's auto-generated one
  assert.ok(
    isSameApp(
      item("Shortcut\\Id", "C:\\lnk\\app.lnk", "C:\\real\\target.exe"),
      win("Window\\Id", "C:\\real\\target.exe"),
    ),
  );
});

Deno.test("window without umid matches by item path", () => {
  assert.ok(isSameApp(item(null, "C:\\Tools\\app.exe"), win(null, "c:\\tools\\APP.exe")));
});

Deno.test("relaunch command matches window process path", () => {
  // pinned via a shortcut: item path is the .lnk, relaunch holds the target
  assert.ok(isSameApp(item(null, "C:\\lnk\\app.lnk", "C:\\real\\target.exe"), win(null, "C:\\real\\target.exe")));
});

Deno.test("relaunch command matches when the window has a umid the item lacks", () => {
  assert.ok(
    isSameApp(item(null, "C:\\real\\target.exe", "C:\\real\\target.exe"), win("Some\\Umid", "C:\\real\\target.exe")),
  );
});

Deno.test("no shared address means different apps", () => {
  assert.ok(!isSameApp(item("A\\id", "C:\\a.exe", "C:\\a.exe"), win("B\\id", "C:\\b.exe")));
});

Deno.test("umid match requires both sides to have one", () => {
  // only the window has an umid: without a shared path this is a different app
  assert.ok(!isSameApp(item(null, "C:\\a.exe"), win("A\\id", "C:\\b.exe")));
  // only the item has an umid, paths differ: different app
  assert.ok(!isSameApp(item("A\\id", "C:\\a.exe"), win(null, "C:\\b.exe")));
});

Deno.test("empty paths never match", () => {
  assert.ok(!isSameApp(item(null, ""), win(null, "")));
  // empty item path with no relaunch: nothing in common
  assert.ok(!isSameApp(item(null, ""), win(null, "C:\\a.exe")));
  // empty item path but the relaunch command is the window's path: matches
  assert.ok(isSameApp(item(null, "", "C:\\a.exe"), win(null, "C:\\a.exe")));
});
