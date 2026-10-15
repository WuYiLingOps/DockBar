import { mount } from "svelte";
import App from "./App.svelte";
import { Widget } from "@dockbar/lib";

import "@dockbar/lib/styles/reset.css";
import "@dockbar/lib/styles/theme/shared.css";
import "@dockbar/lib/styles/theme/dock-preview.css";

const root = document.getElementById("root")!;

await Widget.self.init({
  autoSizeByContent: root,
  hideOnFocusLoss: true,
});

mount(App, {
  target: root,
});
