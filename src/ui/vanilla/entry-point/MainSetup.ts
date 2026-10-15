import type { Widget } from "@dockbar/lib/types";
import { _invoke, webviewInfo } from "src/ui/vanilla/entry-point/_tauri.ts";
import { listen } from "@tauri-apps/api/event";

import { hookLocalStorage } from "src/ui/vanilla/entry-point/_LocalStorage";

// cache-busting: the embedded asset server may serve a stale copy after an
// app update, which leaves widgets running outdated code until the webview
// profile is wiped. Always revalidate.
const indexJsCode = fetch("./index.js", { cache: "reload" }).then((res) => res.text());

// initialize global widget variable, needed by slu-lib
const currentWidgetId = webviewInfo.widgetId;
const widgetList = await _invoke<Widget[]>("state_get_widgets");
window.__SLU_WIDGET = widgetList.find((widget) => widget.id === currentWidgetId)!;

if (!window.__SLU_WIDGET) {
  throw new Error(`Widget definition not found for ${currentWidgetId}`);
}

// reload if widget definition changed
listen<Widget[]>("widgets-changed", ({ payload }) => {
  const actual = payload.find((widget) => widget.id === currentWidgetId);
  if (actual && JSON.stringify(actual) !== JSON.stringify(window.__SLU_WIDGET)) {
    window.location.reload();
  }
});

// set document id
document.documentElement.dataset.widgetId = currentWidgetId;

// hook local storage, to avoid collition of keys
hookLocalStorage(currentWidgetId);

// add base css
const link = document.createElement("link");
link.rel = "stylesheet";
link.href = "/vanilla/entry-point/index.css";
document.head.appendChild(link);

// load index.js
const code = await indexJsCode;
const script = document.createElement("script");
script.type = "module";
script.textContent = code;
document.head.appendChild(script);
