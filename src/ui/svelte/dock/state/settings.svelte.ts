import { DockBarCommand, invoke, RuntimeStyleSheet, Widget } from "@dockbar/lib";
import { Alignment, DockBarSide, HideMode } from "@dockbar/lib/types";
import { isTouchPrimary } from "libs/ui/svelte/utils";
import { locale } from "../i18n/index.ts";
import { declareDocumentAsLayeredHitbox } from "libs/ui/react/utils/layered.ts";
import { systemState } from "./system.svelte.ts";
import { settings as _settings } from "./getters.svelte.ts";
import { dateState } from "libs/ui/svelte/runes/date.svelte.ts";
import { rootEffectAsync } from "libs/ui/svelte/utils/RootEffect.svelte.ts";

let isWidgetReady = $state(false);
const settings = $derived(_settings.value.byWidget["@dockbar/dock"]);

const workArea = $derived(systemState.currentMonitor.rect);

const widgetRect = $derived.by(() => {
  const wa = workArea;
  const hitboxRect = { ...workArea };
  const webviewRect = { ...hitboxRect };

  const scale = systemState.currentMonitor.scaleFactor;
  const size = Math.round(
    (settings.size + settings.padding * 2 + settings.margin * 2) *
      scale,
  );

  /*
   * The webview only needs room for what legitimately escapes the dock bar
   * inside it: hover tooltips (12px gap, ~30px tall, max-width 240px centered
   * on the item — UserApplicationItem.placeTooltip + theme dock.css) and the
   * magnify bulge (up to +100% of the item size). Everything else is dead
   * transparent surface where stray content paints over the desktop (the
   * classic-scrollbar strip bug), so the window hugs the hitbox plus exactly
   * that headroom. Touch has no hover tooltips and keeps the hitbox-sized
   * window it always had.
   */
  const bulge = (settings.iconMagnification ?? 0) > 0 ? settings.size : 0;
  const headroom = Math.round(
    (isHorizontalDock()
      // across the dock axis: bulged item + tooltip gap/height + slack
      ? bulge + 12 + 30 + 16
      // along the dock axis: bulged item center (up to 2x item width) +
      // half of the 240px tooltip + slack
      : settings.margin + settings.padding + settings.size * 2 + 120 + 16) *
      scale,
  );

  switch (settings.position) {
    case DockBarSide.Left:
      hitboxRect.right = hitboxRect.left + size;
      webviewRect.right = isTouchPrimary.value ? hitboxRect.right : Math.min(wa.right, hitboxRect.right + headroom);
      break;
    case DockBarSide.Right:
      hitboxRect.left = hitboxRect.right - size;
      webviewRect.left = isTouchPrimary.value ? hitboxRect.left : Math.max(wa.left, hitboxRect.left - headroom);
      break;
    case DockBarSide.Top:
      hitboxRect.bottom = hitboxRect.top + size;
      webviewRect.bottom = isTouchPrimary.value ? hitboxRect.bottom : Math.min(wa.bottom, hitboxRect.bottom + headroom);
      break;
    case DockBarSide.Bottom:
      hitboxRect.top = hitboxRect.bottom - size;
      webviewRect.top = isTouchPrimary.value ? hitboxRect.top : Math.max(wa.top, hitboxRect.top - headroom);
      break;
  }

  return { hitboxRect, webviewRect };
});

$effect.root(() => {
  $effect(() => {
    locale.set(_settings.value.language);
    dateState.setLang(_settings.value.language);
    dateState.setFormat(_settings.value.dateFormat);
  });
});

export const fullSettings = {
  get value() {
    return _settings.value;
  },
};

class SettingsState {
  popupAlignX = $derived.by(() => {
    switch (settings.position) {
      case DockBarSide.Left:
        return Alignment.Start;
      case DockBarSide.Right:
        return Alignment.End;
      default:
        return Alignment.Center;
    }
  });

  popupAlignY = $derived.by(() => {
    switch (settings.position) {
      case DockBarSide.Bottom:
        return Alignment.End;
      case DockBarSide.Top:
        return Alignment.Start;
      default:
        return Alignment.Center;
    }
  });

  tooltipOrigin = $derived.by(() => {
    const origin: { x: number | null; y: number | null } = { x: null, y: null };
    const rect = settingsState.widgetRect.hitboxRect;
    switch (settingsState.position) {
      case DockBarSide.Left:
        origin.x = rect.right;
        break;
      case DockBarSide.Right:
        origin.x = rect.left;
        break;
      case DockBarSide.Top:
        origin.y = rect.bottom;
        break;
      case DockBarSide.Bottom:
        origin.y = rect.top;
        break;
    }
    return origin;
  });

  get isReady() {
    return isWidgetReady;
  }

  set isReady(v: boolean) {
    isWidgetReady = v;
  }

  get all() {
    return _settings.value;
  }

  get allByWidget() {
    return _settings.value.byWidget;
  }

  get value() {
    return settings;
  }

  get position(): DockBarSide {
    return settings.position;
  }

  get hideMode(): HideMode {
    return settings.hideMode;
  }

  get delayToHide(): number {
    return settings.delayToHide;
  }

  get delayToShow(): number {
    return settings.delayToShow;
  }

  get widgetRect() {
    return widgetRect;
  }
}

export const settingsState = new SettingsState();

export function isHorizontalDock(): boolean {
  const pos = settings.position;
  return pos === DockBarSide.Top || pos === DockBarSide.Bottom;
}

async function updateWidgetPosition() {
  const { hitboxRect, webviewRect } = widgetRect;
  const isTouch = isTouchPrimary.value;
  const hideMode = settings.hideMode;
  const position = settings.position;
  const isReady = settingsState.isReady;

  await Widget.self.setPosition(webviewRect);

  if (!isReady) {
    return;
  }

  if (hideMode === HideMode.Never || isTouch) {
    await invoke(DockBarCommand.RegisterAppBar, {
      rect: hitboxRect,
      edge: position as any,
    });
  } else {
    await invoke(DockBarCommand.UnregisterAppBar);
  }
}

Widget.self.attachPosition();
Widget.self.preReadyTasks.push(rootEffectAsync(updateWidgetPosition));

$effect.root(() => {
  $effect(() => {
    const { size, padding, margin, spaceBetweenItems } = settings;
    const sheet = new RuntimeStyleSheet("@config/dock");
    sheet.addVariable("--config-margin", `${margin}px`);
    sheet.addVariable("--config-padding", `${padding}px`);
    sheet.addVariable("--config-item-size", `${size}px`);
    sheet.addVariable("--config-space-between-items", `${spaceBetweenItems}px`);
    sheet.applyToDocument();
  });

  $effect(() => {
    if (isTouchPrimary.value) return;

    let unlisten: (() => void) | null = null;
    declareDocumentAsLayeredHitbox({
      getPhysicalRect: () => {
        const r = widgetRect.webviewRect;
        return { x: r.left, y: r.top, width: r.right - r.left, height: r.bottom - r.top };
      },
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
    });

    return () => {
      unlisten?.();
    };
  });
});
