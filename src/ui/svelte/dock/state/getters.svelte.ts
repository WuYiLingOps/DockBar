import { DockBarCommand, DockBarEvent, invoke, Settings, subscribe, Widget } from "@dockbar/lib";
import type { UserAppWindowColors } from "@dockbar/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

export const currentMonitorId = Widget.getCurrent().decoded.monitorId!;

export const monitors = lazyRune(() => invoke(DockBarCommand.SystemGetMonitors));
subscribe(DockBarEvent.SystemMonitorsChanged, monitors.setByPayload);

export const mousePos = lazyRune(async () => {
  const [x, y] = await invoke(DockBarCommand.GetMousePosition);
  return { x, y };
});
subscribe(DockBarEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
  mousePos.value = { x, y };
});

export const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));

export const selfWinId = lazyRune(() => invoke(DockBarCommand.GetSelfWindowId));

export const interactables = lazyRune(() => invoke(DockBarCommand.GetUserAppWindows));
subscribe(DockBarEvent.UserAppWindowsChanged, interactables.setByPayload);

export const previews = lazyRune(() => invoke(DockBarCommand.GetUserAppWindowsPreviews));
subscribe(DockBarEvent.UserAppWindowsPreviewsChanged, previews.setByPayload);

export const windowsColors = lazyRune<Record<number, UserAppWindowColors>>(
  () => invoke(DockBarCommand.GetUserAppWindowsColors),
);
subscribe(DockBarEvent.UserAppWindowsColorsChanged, windowsColors.setByPayload);

export const focused = lazyRune(() => invoke(DockBarCommand.GetFocusedApp));

export const widgetStatuses = lazyRune(() => invoke(DockBarCommand.DebugGetWidgetsStatuses));
subscribe(DockBarEvent.WidgetDebugInfoChanged, widgetStatuses.setByPayload);

export const wegItems = lazyRune(() => invoke(DockBarCommand.StateGetDockItems));

await Promise.all([
  monitors.init(),
  mousePos.init(),
  settings.init(),
  selfWinId.init(),
  interactables.init(),
  previews.init(),
  windowsColors.init(),
  focused.init(),
  widgetStatuses.init(),
  wegItems.init(),
]);
