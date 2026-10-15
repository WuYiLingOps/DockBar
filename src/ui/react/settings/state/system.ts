import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { DockBarCommand, DockBarEvent, invoke, subscribe, UIColors } from "@dockbar/lib";
import { effect, signal } from "@preact/signals";

export const uiColors = lazySignal(() => invoke(DockBarCommand.SystemGetColors));
subscribe(DockBarEvent.ColorsChanged, uiColors.setByPayload);
await uiColors.init();

effect(() => {
  new UIColors(uiColors.value).setAsCssVariables();
});

export const monitors = lazySignal(() => invoke(DockBarCommand.SystemGetMonitors));
subscribe(DockBarEvent.SystemMonitorsChanged, monitors.setByPayload);
await monitors.init();

export const autostart = signal(await invoke(DockBarCommand.GetAutoStartStatus));
export async function setAutoStart(enabled: boolean) {
  await invoke(DockBarCommand.SetAutoStart, { enabled });
  autostart.value = enabled;
}
