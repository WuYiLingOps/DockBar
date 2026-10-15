import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@dockbar/lib";
import { LogicalSize } from "@dockbar/lib/tauri";

import "@dockbar/lib/styles/reset.css";
import "@dockbar/lib/styles/theme/shared.css";

await Widget.self.init();
await Promise.all([
  Widget.self.window.setSizeConstraints({ minWidth: 600, minHeight: 400 }),
  Widget.self.window.setSize(new LogicalSize(800, 500)),
  Widget.self.window.center(),
]);

mount(App, {
  target: document.getElementById("root")!,
});
