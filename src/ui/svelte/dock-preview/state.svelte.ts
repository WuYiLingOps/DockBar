import { DockBarCommand, DockBarEvent, invoke, subscribe, Widget } from "@dockbar/lib";
import { DockBarSide, type UserAppWindow } from "@dockbar/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

const interactables = lazyRune<UserAppWindow[]>(() => invoke(DockBarCommand.GetUserAppWindows));
subscribe(DockBarEvent.UserAppWindowsChanged, interactables.setByPayload);

const previews = lazyRune<Record<number, { data: string }>>(() => invoke(DockBarCommand.GetUserAppWindowsPreviews));
subscribe(DockBarEvent.UserAppWindowsPreviewsChanged, previews.setByPayload);

await Promise.all([interactables.init(), previews.init()]);

let hwnds = $state<number[]>([]);
let position = $state<DockBarSide>(DockBarSide.Bottom);

Widget.self.onTrigger(({ customArgs }) => {
  hwnds = (customArgs?.hwnds as number[]) ?? [];
  position = (customArgs?.position as DockBarSide) ?? DockBarSide.Bottom;
});

const currentInteractables = $derived(interactables.value?.filter((w) => hwnds.includes(w.hwnd)) ?? []);

class PreviewState {
  get currentInteractables() {
    return currentInteractables;
  }

  get previews() {
    return previews;
  }

  get position() {
    return position;
  }
}

export const previewState = new PreviewState();
