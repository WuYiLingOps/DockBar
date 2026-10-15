import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { DockBarCommand, DockBarEvent, invoke, subscribe } from "@dockbar/lib";
import type { ResourceText } from "@dockbar/lib/types";
import { computed } from "@preact/signals";
import { getResourceText } from "@shared";
import { language } from "./mod";

function SorterByDisplayName(
  a: { metadata: { displayName: ResourceText } },
  b: { metadata: { displayName: ResourceText } },
): number {
  const aDisplayName = getResourceText(a.metadata.displayName, language.value);
  const bDisplayName = getResourceText(b.metadata.displayName, language.value);

  return aDisplayName.localeCompare(bDisplayName, language.value);
}

const _widgets = lazySignal(() => invoke(DockBarCommand.StateGetWidgets));
subscribe(DockBarEvent.StateWidgetsChanged, _widgets.setByPayload);
await _widgets.init();

const _iconPacks = lazySignal(() => invoke(DockBarCommand.StateGetIconPacks));
subscribe(DockBarEvent.StateIconPacksChanged, _iconPacks.setByPayload);
await _iconPacks.init();

export const widgets = computed(() => _widgets.value.toSorted(SorterByDisplayName));
export const iconPacks = computed(() => _iconPacks.value.toSorted(SorterByDisplayName));
