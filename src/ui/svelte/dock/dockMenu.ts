import { DockBarCommand, invoke, Widget } from "@dockbar/lib";
import { Alignment } from "@dockbar/lib/types";
import { dialog } from "@dockbar/lib/tauri";
import type { ContextMenu, ContextMenuCallbackPayload, WidgetId } from "@dockbar/lib/types";
import { dockState, dockStateActions } from "./state/items.svelte.ts";
import { isHorizontalDock, settingsState } from "./state/settings.svelte.ts";

const MENU_ID = crypto.randomUUID();

const MENU_EVENT = "dock::bar_menu_click";

let _t: (key: string) => string = (key) => key;

async function handleBarMenuClick({ key, value }: ContextMenuCallbackPayload) {
  switch (key) {
    case "add-separator":
      dockStateActions.addSeparatorNear(value as { x: number; y: number });
      break;
    case "reorder":
      dockState.state = {
        ...dockState.state,
        isReorderDisabled: !dockState.isReorderDisabled,
      };
      break;
    case "task_manager":
      invoke(DockBarCommand.OpenFile, { path: "Taskmgr.exe" });
      break;
    case "settings":
      invoke(DockBarCommand.TriggerWidget, {
        payload: { id: "@dockbar/settings" as WidgetId },
      });
      break;
    case "add-item": {
      const files = await dialog.open({
        title: _t("taskbar_menu.add_file"),
        multiple: true,
        filters: [
          { name: "lnk", extensions: ["lnk"] },
          { name: "*", extensions: ["*"] },
        ],
      });
      for (const path of files || []) {
        await invoke(DockBarCommand.WegPinItem, { path });
      }
      break;
    }
    case "add-apps": {
      // open the picker above the dock: centered on the dock's hitbox, bottom
      // edge sitting on the dock (End align on the dock's top edge), so the
      // grid never overlaps the bar and fitIntoMonitor keeps it on-screen
      const { widgetRect, popupAlignX, popupAlignY } = settingsState;
      const hitbox = widgetRect.hitboxRect;
      const desiredPosition = {
        x: Math.round((hitbox.left + hitbox.right) / 2),
        // `i32::MIN` marks an unset axis (falls back to the cursor), so pass a
        // real anchor: the dock edge the picker should grow away from.
        y: popupAlignY === Alignment.End ? hitbox.top : hitbox.bottom,
      };
      invoke(DockBarCommand.TriggerWidget, {
        payload: {
          id: "@dockbar/app-picker" as WidgetId,
          desiredPosition,
          alignX: popupAlignX,
          alignY: popupAlignY,
        },
      });
      break;
    }
  }
}

Widget.self.webview.listen<ContextMenuCallbackPayload>(MENU_EVENT, ({ payload }) => {
  handleBarMenuClick(payload);
});

export function getDockBarMenu(
  t: (key: string) => string,
  cursor: { x: number; y: number },
): ContextMenu {
  _t = t;
  const { isReorderDisabled } = dockState;

  return {
    identifier: MENU_ID,
    items: [
      {
        type: "Item",
        key: "add-apps",
        icon: "MdOutlineApps",
        label: t("taskbar_menu.add_apps"),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "add-item",
        icon: "RiFileAddLine",
        label: t("taskbar_menu.add_file"),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "add-separator",
        icon: isHorizontalDock() ? "LuSeparatorVertical" : "LuSeparatorHorizontal",
        label: t("taskbar_menu.add_separator"),
        value: cursor,
        callbackEvent: MENU_EVENT,
      },
      { type: "Separator" },
      {
        type: "Item",
        key: "reorder",
        icon: isReorderDisabled ? "CgLockUnlock" : "CgLock",
        label: t(
          isReorderDisabled ? "context_menu.reorder_enable" : "context_menu.reorder_disable",
        ),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "task_manager",
        icon: "PiChartLineFill",
        label: t("taskbar_menu.task_manager"),
        callbackEvent: MENU_EVENT,
      },
      {
        type: "Item",
        key: "settings",
        icon: "RiSettings4Fill",
        label: t("taskbar_menu.settings"),
        callbackEvent: MENU_EVENT,
      },
    ],
  };
}
