import { DockBarCommand, invoke } from "@dockbar/lib";
import { Alignment, DockBarSide, type UserAppWindow, type WidgetId } from "@dockbar/lib/types";
import { settingsState } from "./state/settings.svelte.ts";

export function triggerPreviewWidget(itemEl: HTMLElement, windows: UserAppWindow[]) {
  const dockSide = settingsState.position;

  const elRect = itemEl.getBoundingClientRect();
  const viewRect = settingsState.widgetRect.hitboxRect;

  const toPhysical = (n: number) => Math.round(n * globalThis.devicePixelRatio);

  let x: number;
  let y: number;
  let alignX: Alignment;
  let alignY: Alignment;

  switch (dockSide) {
    case DockBarSide.Bottom:
      x = viewRect.left + toPhysical(elRect.left + elRect.width / 2);
      y = viewRect.top;
      alignX = Alignment.Center;
      alignY = Alignment.End;
      break;
    case DockBarSide.Top:
      x = viewRect.left + toPhysical(elRect.left + elRect.width / 2);
      y = viewRect.bottom;
      alignX = Alignment.Center;
      alignY = Alignment.Start;
      break;
    case DockBarSide.Left:
      x = viewRect.right;
      y = viewRect.top + toPhysical(elRect.top + elRect.height / 2);
      alignX = Alignment.Start;
      alignY = Alignment.Center;
      break;
    case DockBarSide.Right:
      x = viewRect.left;
      y = viewRect.top + toPhysical(elRect.top + elRect.height / 2);
      alignX = Alignment.End;
      alignY = Alignment.Center;
      break;
  }

  invoke(DockBarCommand.TriggerWidget, {
    payload: {
      id: "@dockbar/dock-preview" as WidgetId,
      desiredPosition: { x, y },
      alignX,
      alignY,
      customArgs: {
        hwnds: windows.map((w) => w.hwnd),
        position: dockSide,
      },
    },
  });
}
