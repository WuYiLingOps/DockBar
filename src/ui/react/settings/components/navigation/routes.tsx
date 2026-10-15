import { Icon } from "libs/ui/react/components/Icon";
import type React from "react";

export enum RoutePath {
  General = "/general",
  Shortcuts = "/shortcuts",
  SettingsByMonitor = "/monitors",
  DevTools = "/developer",
}

export const RouteIcons: { [key in RoutePath]?: React.ReactNode } = {
  [RoutePath.General]: <Icon iconName="RiSettings3Fill" />,
  [RoutePath.SettingsByMonitor]: <Icon iconName="PiMonitorBold" />,
  [RoutePath.Shortcuts]: <Icon iconName="MdLaunch" />,
  [RoutePath.DevTools]: <Icon iconName="PiCodeBold" />,
};
