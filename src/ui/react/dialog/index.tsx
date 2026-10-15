import { Widget } from "@dockbar/lib";
import { getRootContainer } from "libs/ui/react/utils/index";
import { createRoot } from "react-dom/client";

import { App } from "./app.tsx";

import "@dockbar/lib/styles/reset.css";
import "@dockbar/lib/styles/theme/shared.css";
import "./global.css";

await Widget.self.init();

const container = getRootContainer();
createRoot(container).render(<App />);
