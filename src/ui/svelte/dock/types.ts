import type { DockItem } from "@dockbar/lib/types";

export type AppOrFileDockItem = Extract<DockItem, { type: "AppOrFile" }>;
export type SeparatorDockItem = Extract<DockItem, { type: "Separator" }>;

/** @alias */
export type SwItem = DockItem;
