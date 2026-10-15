import { Route, Routes } from "react-router";

import { SettingsByMonitor } from "./modules/ByMonitor/infra/index.tsx";
import { DeveloperTools } from "./modules/developer/infra.tsx";
import { General } from "./modules/general/infra/index.tsx";
import { Shortcuts } from "./modules/shortcuts/infrastructure.tsx";

import { Layout } from "./components/layout/index.tsx";
import { RoutePath } from "./components/navigation/routes.tsx";
import { WidgetView } from "./modules/resources/Widget/View.tsx";

export function Routing() {
  return (
    <Routes>
      <Route Component={Layout}>
        <Route index Component={General} />
        <Route path={RoutePath.General} Component={General} />
        <Route path={RoutePath.SettingsByMonitor} Component={SettingsByMonitor} />
        <Route path={RoutePath.Shortcuts} Component={Shortcuts} />
        <Route path={RoutePath.DevTools} Component={DeveloperTools} />
        <Route path="widget" Component={WidgetView} />
      </Route>
    </Routes>
  );
}
