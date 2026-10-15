import { DockBarCommand, invoke } from "@dockbar/lib";
import { settings } from "../../../../state/mod";
import type { DockBarSettings } from "@dockbar/lib/types";

/**
 * Patches the DockBar configuration with partial updates.
 * This helper function simplifies updating the dock settings by handling
 * the nested structure automatically.
 *
 * @example
 * patchWegConfig({ enabled: true, margin: 10 });
 */
export function patchWegConfig(patch: Partial<DockBarSettings>) {
  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      "@dockbar/dock": {
        ...settings.value.byWidget["@dockbar/dock"],
        ...patch,
      },
    },
  };
}

/**
 * Gets the current DockBar configuration
 */
export function getWegConfig(): DockBarSettings {
  return settings.value.byWidget["@dockbar/dock"];
}

/**
 * Imports pinned items from Windows taskbar
 */
export async function importFromWindowsTaskbar(): Promise<number> {
  return await invoke(DockBarCommand.WegImportPinnedTaskbarItems);
}
