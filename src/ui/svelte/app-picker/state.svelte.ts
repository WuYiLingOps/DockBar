import { DockBarCommand, invoke, Widget } from "@dockbar/lib";
import type { DockItem, StartMenuEntry } from "@dockbar/lib/types";
import { isSameApp } from "libs/ui/dockItemMatching.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let apps = $state<StartMenuEntry[]>([]);
let loading = $state(true);
let search = $state("");
let dockItems = $state<DockItem[]>([]);

const settings = lazyRune(() => import("@dockbar/lib").then(({ Settings }) => Settings.getAsync()));

const filteredApps = $derived.by(() => {
  const query = search.trim().toLowerCase();
  const seen = new Set<string>();
  const result: StartMenuEntry[] = [];

  for (const app of apps) {
    if (query && !app.displayName.toLowerCase().includes(query)) continue;

    // start-menu dedupe already ran on the backend, but both a shortcut and a
    // packaged entry can still resolve to the same dock item, so filter again
    // by what would land on the dock.
    const key = (app.umid ?? app.path.toString()).toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);

    result.push(app);
  }

  return result;
});

class AppPickerState {
  get loading() {
    return loading;
  }
  get search() {
    return search;
  }
  set search(value: string) {
    search = value;
  }
  get filtered() {
    return filteredApps;
  }
}

export const appPickerState = new AppPickerState();

export { settings };

/** Whether the app is already on the dock (pinned or just open). */
export function isAlreadyOnDock(app: StartMenuEntry): boolean {
  return dockItems.some(
    (item) =>
      item.type === "AppOrFile" &&
      isSameApp(
        { umid: item.umid, path: item.path, relaunchCommand: item.relaunch?.command ?? null },
        { umid: app.umid, path: app.path.toString() },
      ),
  );
}

/** Pins the app on the dock through the same backend path used by drag & drop. */
export async function pinApp(app: StartMenuEntry): Promise<void> {
  if (isAlreadyOnDock(app)) {
    return;
  }

  await invoke(DockBarCommand.WegPinItem, {
    path: app.path.toString(),
    umid: app.umid,
  });
  // dock_pin_item emits DockAddItem, which the dock consumes to pin/insert the item.
  dockItems = [
    ...dockItems,
    {
      type: "AppOrFile",
      id: crypto.randomUUID(),
      displayName: app.displayName,
      umid: app.umid,
      path: app.path.toString(),
      pinned: true,
      preventPinning: false,
      relaunch: null,
    },
  ];
}

/** Hides the picker (popup widgets also hide on focus loss; this is for explicit close). */
export function closePicker(): void {
  Widget.getCurrent().hide();
}

async function loadApps(): Promise<void> {
  try {
    const [dockState, items] = await Promise.all([
      invoke(DockBarCommand.StateGetDockItems),
      invoke(DockBarCommand.GetStartMenuItems),
    ]);
    dockItems = [...dockState.left, ...dockState.center, ...dockState.right];
    apps = items;
  } finally {
    loading = false;
  }
}

await settings.init();

// prefetch so the grid is populated on the first open; later triggers refresh
loadApps();

Widget.self.onTrigger(() => {
  loadApps();
});
