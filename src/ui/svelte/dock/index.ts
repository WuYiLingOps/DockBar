import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import App from "./App.svelte";
import { DockBarCommand, Widget } from "@dockbar/lib";
import { invoke } from "@tauri-apps/api/core";

import "./styles/variables.css";
import "@dockbar/lib/styles/reset.css";
import "@dockbar/lib/styles/theme/shared.css";
import "@dockbar/lib/styles/theme/dock.css";
import "./styles/global.css";

await Widget.getCurrent().init();

mount(App, {
  target: getRootContainer(),
});

Widget.self.window.onDragDropEvent(async (e) => {
  if (e.payload.type === "drop") {
    for (const path of e.payload.paths) {
      await invoke(DockBarCommand.WegPinItem, { path });
    }
  }
});
