import { mount } from "svelte";
import App from "./app.svelte";
import { Widget } from "@dockbar/lib";

import "@dockbar/lib/styles/reset.css";
import "@dockbar/lib/styles/theme/shared.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
});

mount(App, {
  target: root,
});
