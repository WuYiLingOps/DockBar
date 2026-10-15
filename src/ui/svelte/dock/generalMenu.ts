import { Widget } from "@dockbar/lib";
import type { ContextMenu, ContextMenuCallbackPayload, ContextMenuItem } from "@dockbar/lib/types";
import type { SwItem } from "./types.ts";
import { dockStateActions, HARDCODED_SEPARATOR_LEFT, HARDCODED_SEPARATOR_RIGHT } from "./state/items.svelte.ts";

const identifier = crypto.randomUUID();
const onItemMenuClick = "dock::item_menu_click";

let pendingItem: SwItem | null = null;

Widget.self.webview.listen<ContextMenuCallbackPayload>(onItemMenuClick, ({ payload }) => {
  const { key } = payload;
  const item = pendingItem;
  if (!item) return;

  if (key === "remove") {
    dockStateActions.remove(item.id);
  }
});

export function getMenuForItem(t: (key: string) => string, item: SwItem): ContextMenu {
  pendingItem = item;

  if (
    item.type === "Separator" &&
    item.id !== HARDCODED_SEPARATOR_LEFT.id &&
    item.id !== HARDCODED_SEPARATOR_RIGHT.id
  ) {
    const items: ContextMenuItem[] = [
      {
        type: "Item",
        key: "remove",
        icon: "IoRemove",
        label: t("context_menu.remove_separator"),
        callbackEvent: onItemMenuClick,
      },
    ];

    return { identifier, items };
  }

  return { identifier, items: [] };
}
