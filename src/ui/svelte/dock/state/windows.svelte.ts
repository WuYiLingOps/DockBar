import { DockBarEvent, subscribe, Widget } from "@dockbar/lib";
import {
  DockBarSide,
  type FocusedApp,
  type UserAppWindow,
  type UserAppWindowColors,
  WegTemporalItemsVisibility,
} from "@dockbar/lib/types";
import { settingsState } from "./settings.svelte.ts";
import { debounce } from "lodash";
import type { AppOrFileDockItem } from "../types.ts";
import { focused, interactables, previews, selfWinId, widgetStatuses, windowsColors } from "./getters.svelte.ts";
import { isSameApp } from "libs/ui/dockItemMatching.ts";

export { focused, interactables, previews, widgetStatuses, windowsColors };

const widget = Widget.getCurrent();

let _delayedFocused = $state<FocusedApp | null>(null);
const setDelayedFocused = debounce((v: FocusedApp) => {
  _delayedFocused = v;
}, 200);

subscribe(DockBarEvent.GlobalFocusChanged, (e) => {
  focused.value = e.payload;
  setDelayedFocused(e.payload);
  if (e.payload.hwnd !== selfWinId.value) {
    setDelayedFocused.flush();
  }
});

/** windows that belong to this dock, honoring the "On Monitor" temporal items visibility */
const _dockWindows = $derived.by(() => {
  if (settingsState.value.temporalItemsVisibility !== WegTemporalItemsVisibility.OnMonitor) {
    return interactables.value;
  }
  return interactables.value.filter((w) => w.monitor === widget.decoded.monitorId);
});

const _topInteractableWindow = $derived(
  interactables.value
    .toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt)
    .find((w) => w.monitor === widget.decoded.monitorId && !w.isIconic),
);

const _currentMonitorMaximizedColors = $derived.by((): UserAppWindowColors | null => {
  const monitorId = widget.decoded.monitorId;
  const maximized = interactables.value
    .toSorted((a, b) => b.lastForegroundAt - a.lastForegroundAt)
    .find((w) => !w.isIconic && w.isZoomed && w.monitor === monitorId);
  if (!maximized) return null;
  return windowsColors.value[maximized.hwnd] ?? null;
});

const _isDockOverlapped = $derived.by(() => {
  // If foreground is not in interactable windows, return false directly, this handled start menu or desktop focus cases.
  const foreground = focused.value;
  if (!interactables.value.some((w) => w.hwnd === foreground.hwnd)) {
    return false;
  }

  // Check if any interactable window overlaps with the hitbox
  const a = settingsState.widgetRect.hitboxRect;
  for (const app of interactables.value) {
    if (app.monitor !== widget.decoded.monitorId || app.isIconic || !app.rect) continue;
    const b = app.rect;

    if (!(a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom)) {
      return true;
    }
  }

  return false;
});

class WindowsState {
  get dockWindows() {
    return _dockWindows;
  }

  get topInteractableWindow() {
    return _topInteractableWindow;
  }

  get delayedFocused(): FocusedApp | null {
    return _delayedFocused;
  }

  get isDockOverlapped() {
    return _isDockOverlapped;
  }
}

export const windowsState = new WindowsState();

// on change of this function update src\background\widgets\dock\cli.rs too.
export function getWindowsForItem(
  item: AppOrFileDockItem,
  windows: UserAppWindow[],
): UserAppWindow[] {
  return windows.filter((w) =>
    isSameApp(
      { umid: item.umid, path: item.path, relaunchCommand: item.relaunch?.command ?? null },
      { umid: w.umid, path: w.process.path?.toString() ?? "" },
    )
  );
}

$effect.root(() => {
  $effect(() => {
    const colors = _currentMonitorMaximizedColors;
    const root = document.documentElement;

    root.dataset.thereIsMaximizedOnBg = `${!!colors}`;

    if (!colors) {
      root.style.removeProperty("--window-gradient");
      return;
    }

    const toRgba = ({ r, g, b, a }: { r: number; g: number; b: number; a: number }) =>
      `rgba(${r},${g},${b},${(a / 255).toFixed(3)})`;

    const toHGradient = (stops: typeof colors.top) =>
      `linear-gradient(to right,${
        stops
          .map((c, i) => `${toRgba(c)} ${((i / (stops.length - 1)) * 100).toFixed(1)}%`)
          .join(",")
      })`;

    const toVGradient = (stops: typeof colors.left) =>
      `linear-gradient(to bottom,${
        stops
          .map((c, i) => `${toRgba(c)} ${((i / (stops.length - 1)) * 100).toFixed(1)}%`)
          .join(",")
      })`;

    const pos = settingsState.position;
    if (pos === DockBarSide.Top) {
      root.style.setProperty("--window-gradient", toHGradient(colors.top));
    } else if (pos === DockBarSide.Bottom) {
      root.style.setProperty("--window-gradient", toHGradient(colors.bottom));
    } else if (pos === DockBarSide.Left) {
      root.style.setProperty("--window-gradient", toVGradient(colors.left));
    } else {
      root.style.setProperty("--window-gradient", toVGradient(colors.right));
    }
  });
});
