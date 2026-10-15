import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { createRoot } from "react-dom/client";
import { I18nextProvider } from "react-i18next";
import { HashRouter } from "react-router";
import { Widget } from "@dockbar/lib";
import { LogicalSize } from "@dockbar/lib/tauri";

import { App } from "./app.tsx";

import i18n from "./i18n/index.ts";

import "./styles/variables.css";
import "@dockbar/lib/styles/reset.css";
import "@dockbar/lib/styles/theme/shared.css";
import "./styles/global.css";
import "./styles/RichText.css";

const { window } = Widget.self;

await Promise.all([
  window.setDecorations(false),
  window.setSizeConstraints({ minWidth: 600, minHeight: 400 }),
  window.setSize(new LogicalSize(800, 500)),
]);
await window.center();

await Widget.self.init();
await Widget.self.show();

Widget.self.onTrigger((payload) => {
  window.unminimize();
  window.setFocus();
  const route = payload.customArgs?.["route"] as string | undefined;
  if (route) {
    globalThis.location.hash = route;
  }
});

const container = getRootContainer();
createRoot(container).render(
  <I18nextProvider i18n={i18n}>
    <HashRouter>
      <App />
    </HashRouter>
  </I18nextProvider>,
);
