import { DockBarCommand, DockBarEvent, invoke, subscribe } from "@dockbar/lib";
import type { DockItem, DockItems } from "@dockbar/lib/types";
import { debounce } from "lodash";
import { emit, listen } from "@tauri-apps/api/event";
import type { AppOrFileDockItem, SeparatorDockItem } from "../types.ts";
import { getWindowsForItem, interactables } from "./windows.svelte.ts";
import { stableUUID } from "libs/ui/utils.ts";
import { isSameApp } from "libs/ui/dockItemMatching.ts";
import { wegItems } from "./getters.svelte.ts";
import { isHorizontalDock } from "./settings.svelte.ts";

interface OptimisticDockState {
  isReorderDisabled: boolean;
  items: DockItem[];
}

interface SyncPayload {
  source: string;
  state: OptimisticDockState;
}

const CLIENT_ID = crypto.randomUUID();

export const HARDCODED_SEPARATOR_LEFT: SeparatorDockItem = {
  id: "hardcoded-separator-1",
  type: "Separator",
};

export const HARDCODED_SEPARATOR_RIGHT: SeparatorDockItem = {
  id: "hardcoded-separator-2",
  type: "Separator",
};

function getStateFromStored(state: DockItems): OptimisticDockState {
  // check used for duplicate ids
  const seen = new Set<string>([HARDCODED_SEPARATOR_LEFT.id, HARDCODED_SEPARATOR_RIGHT.id]);
  const sanitize = (items: DockItem[]) =>
    items.filter((item) => {
      if (seen.has(item.id)) return false;
      seen.add(item.id);
      return true;
    });

  return {
    isReorderDisabled: state.isReorderDisabled,
    items: [
      ...sanitize(state.left),
      HARDCODED_SEPARATOR_LEFT,
      ...sanitize(state.center),
      HARDCODED_SEPARATOR_RIGHT,
      ...sanitize(state.right),
    ],
  };
}

let _dockState = $state(getStateFromStored(wegItems.value));

export const dockState = {
  get isReorderDisabled() {
    return _dockState.isReorderDisabled;
  },
  get items() {
    return _dockState.items;
  },
  get state() {
    return _dockState;
  },
  set state(value: OptimisticDockState) {
    _dockState = value;
  },
  set items(value: DockItem[]) {
    _dockState = { ..._dockState, items: value };
  },
};

subscribe(DockBarEvent.DockAddItem, (e) => {
  const item: AppOrFileDockItem = {
    ...e.payload,
    id: crypto.randomUUID(),
    type: "AppOrFile",
  };

  // the app could already be on the dock, pinned or only open, so it is pinned instead of
  // being added again.
  const existing = _dockState.items.find(
    (i): i is AppOrFileDockItem => i.type === "AppOrFile" && isSameAppOrFile(i, item),
  );
  if (existing) {
    if (!existing.pinned) {
      dockStateActions.pinApp(existing.id);
    }
    return;
  }

  const items = insertIntoPinnedZone(_dockState.items, item);
  _dockState = { ..._dockState, items };
});

/** Inserts an item at the end of the pinned zone, before the visible divider. */
function insertIntoPinnedZone(items: DockItem[], item: DockItem): DockItem[] {
  const next = items.filter((i) => i.id !== item.id);
  let insertIdx = next.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
  if (insertIdx === -1) insertIdx = next.length;
  let previous = next[insertIdx - 1];
  while (
    insertIdx > 0 &&
    previous?.type === "Separator" &&
    previous.id !== HARDCODED_SEPARATOR_LEFT.id
  ) {
    insertIdx--;
    previous = next[insertIdx - 1];
  }
  next.splice(insertIdx, 0, item);
  return next;
}

/** Inserts an item at the start of the temporal zone, right of the divider. */
function insertIntoTemporalZone(items: DockItem[], item: DockItem): DockItem[] {
  const next = items.filter((i) => i.id !== item.id);
  const rightIdx = next.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
  next.splice(rightIdx === -1 ? next.length : rightIdx + 1, 0, item);
  return next;
}

subscribe(DockBarEvent.DockItemsChanged, (e) => {
  // Reload the entire dock state from the backend
  _dockState = getStateFromStored(e.payload);
});

let isRemoteUpdate = false;
listen<SyncPayload>("hidden::sync-dock-items", ({ payload }) => {
  if (payload.source === CLIENT_ID) return;
  if (JSON.stringify(payload.state) !== JSON.stringify(_dockState)) {
    isRemoteUpdate = true;
    _dockState = payload.state;
  }
});

const emitSyncEvent = debounce((state: OptimisticDockState) => {
  emit<SyncPayload>("hidden::sync-dock-items", { source: CLIENT_ID, state });
}, 300);

/** Identity view of an item/window shared by the dedup helpers below. */
function identityOf(item: AppOrFileDockItem) {
  return { umid: item.umid, path: item.path, relaunchCommand: item.relaunch?.command ?? null };
}

/** Same app by umid or any exe-path coincidence (item path, relaunch command). */
export function isSameAppOrFile(a: AppOrFileDockItem, b: AppOrFileDockItem): boolean {
  return isSameApp(identityOf(a), identityOf(b));
}

export function listToGroups(items: DockItem[], includeSeparator = false) {
  let idx1 = items.findIndex((i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_LEFT.id);
  let idx2 = items.findIndex((i) => typeof i !== "string" && i.id === HARDCODED_SEPARATOR_RIGHT.id);
  if (idx1 > idx2) {
    [idx1, idx2] = [idx2, idx1];
  }
  return {
    left: items.slice(0, idx1),
    center: includeSeparator ? items.slice(idx1, idx2 + 1) : items.slice(idx1 + 1, idx2),
    right: items.slice(idx2 + 1),
  };
}

const saveDockState = debounce(async (state: OptimisticDockState) => {
  console.trace("Saving dock state");
  await invoke(DockBarCommand.StateWriteDockItems, {
    items: {
      isReorderDisabled: state.isReorderDisabled,
      ...listToGroups(state.items),
    },
  });
}, 1000);

let mounted = false;
$effect.root(() => {
  $effect(() => {
    const state = _dockState;

    if (!mounted) {
      mounted = true;
      return;
    }

    if (isRemoteUpdate) {
      isRemoteUpdate = false;
      return;
    }

    emitSyncEvent(state);
    saveDockState(state);
  });
});

export const dockStateActions = {
  remove(idToRemove: string) {
    _dockState = {
      ..._dockState,
      items: _dockState.items.filter((item) => item.id !== idToRemove),
    };
  },
  pinApp(id: string) {
    const items = _dockState.items;
    const index = items.findIndex((item) => item.id === id);
    if (index === -1) return;
    const item = items[index]!;
    if (item.type !== "AppOrFile") return;
    const pinned: AppOrFileDockItem = { ...item, pinned: true };
    const rightIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
    // a running app being pinned moves from the temporal zone into the
    // pinned zone, so it ends up left of the visible divider
    const next = rightIdx !== -1 && index > rightIdx
      ? insertIntoPinnedZone(items, pinned)
      : items.map((item) => (item.id === id ? pinned : item));
    _dockState = { ..._dockState, items: next };
  },
  unpinApp(id: string) {
    const items = _dockState.items;
    const index = items.findIndex((item) => item.id === id);
    if (index === -1) return;
    const item = items[index]!;
    if (item.type !== "AppOrFile") return;
    const unpinned: AppOrFileDockItem = { ...item, pinned: false };
    const rightIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);
    // keep the temporal zone to the right of the divider when unpinning
    const next = rightIdx !== -1 && index < rightIdx
      ? insertIntoTemporalZone(items, unpinned)
      : items.map((item) => (item.id === id ? unpinned : item));
    _dockState = { ..._dockState, items: next };
  },
  /** Inserts a new separator next to whichever rendered item is closest to the given cursor position. */
  addSeparatorNear(cursor: { x: number; y: number }) {
    const newSeparator: SeparatorDockItem = { id: crypto.randomUUID(), type: "Separator" };
    const items = [..._dockState.items];

    const containers = Array.from(
      document.querySelectorAll<HTMLElement>(".dock-item-drag-container"),
    );

    let insertIdx = items.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);

    if (containers.length > 0) {
      const horizontal = isHorizontalDock();
      let nearestId: string | undefined;
      let nearestDist = Infinity;
      let insertAfter = false;

      for (const el of containers) {
        const rect = el.getBoundingClientRect();
        const cx = rect.left + rect.width / 2;
        const cy = rect.top + rect.height / 2;
        const dist = Math.hypot(cursor.x - cx, cursor.y - cy);
        if (dist < nearestDist) {
          nearestDist = dist;
          nearestId = el.dataset.itemId;
          insertAfter = horizontal ? cursor.x > cx : cursor.y > cy;
        }
      }

      const nearestIdx = items.findIndex((i) => i.id === nearestId);
      if (nearestIdx !== -1) {
        insertIdx = insertAfter ? nearestIdx + 1 : nearestIdx;
      }
    }

    items.splice(insertIdx, 0, newSeparator);
    _dockState = { ..._dockState, items };
  },
};

$effect.root(() => {
  $effect(() => {
    const windows = interactables.value;
    const state = _dockState;

    const appOrFileItems = state.items.filter(
      (item): item is AppOrFileDockItem => item.type === "AppOrFile",
    );

    // drop unpinned temporal items whose app no longer has windows
    const itemsToRemove = new Set(
      appOrFileItems
        .filter((item) => !item.pinned && getWindowsForItem(item, windows).length === 0)
        .map((item) => item.id),
    );

    // running windows not covered by any item become temporal items, placed on
    // the other side of the right separator (kept apart from pinned items)
    const remainingItems = appOrFileItems.filter((item) => !itemsToRemove.has(item.id));
    const uncoveredWindows = windows.filter(
      (w) => !remainingItems.some((item) => getWindowsForItem(item, [w]).length > 0),
    );

    const seen = new Set<string>();
    const newItems: AppOrFileDockItem[] = [];

    for (const w of uncoveredWindows) {
      const key = w.umid ?? w.process.path?.toString();
      if (!key) continue;
      if (seen.has(key)) continue;
      seen.add(key);

      newItems.push({
        id: stableUUID(key),
        type: "AppOrFile",
        displayName: w.appName,
        umid: w.umid ?? null,
        path: w.process.path?.toString() ?? "",
        pinned: false,
        preventPinning: w.preventPinning,
        relaunch: w.relaunch ?? null,
      });
    }

    if (itemsToRemove.size === 0 && newItems.length === 0) return;

    const filteredItems = state.items.filter((item) => !itemsToRemove.has(item.id));
    const separatorRightIdx = filteredItems.findIndex((i) => i.id === HARDCODED_SEPARATOR_RIGHT.id);

    // Within the temporal zone (past the right separator), items backed by a
    // pinned dock item go first (right after the separator), the unpinned ones
    // follow in their previous order.
    const inTemporalZone = (item: DockItem) => filteredItems.indexOf(item) > separatorRightIdx;
    const isPinnedBacked = (item: DockItem) =>
      item.type === "AppOrFile" &&
      appOrFileItems.some((p) => p.pinned && p.id !== item.id && isSameAppOrFile(p, item));

    const temporalZone = filteredItems.filter(inTemporalZone);
    const temporalZoneSorted = [
      ...temporalZone.filter(isPinnedBacked),
      ...temporalZone.filter((item) => !isPinnedBacked(item)),
    ];

    _dockState = {
      ...state,
      items: [
        ...filteredItems.slice(0, separatorRightIdx + 1),
        ...newItems,
        ...temporalZoneSorted,
      ],
    };
  });
});
