import type { AppOrFileDockItem } from "../types";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { UserAppWindow } from "@dockbar/lib/types";

type Badge = { type: "image"; src: string };

export function getBadge(item: AppOrFileDockItem, windows: UserAppWindow[]): Badge | null {
  void item;

  const winWithBadge = windows
    .toSorted((a, b) => b.badgeUpdatedAt - a.badgeUpdatedAt)
    .find((a) => a.badgeIconPath);
  if (winWithBadge && winWithBadge.badgeIconPath && winWithBadge.badgeUpdatedAt) {
    return {
      type: "image",
      src: convertFileSrc(winWithBadge.badgeIconPath) + `?v=${winWithBadge.badgeUpdatedAt}`,
    };
  }

  return null;
}
