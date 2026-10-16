<script lang="ts">
  import { invoke, DockBarCommand } from "@dockbar/lib";
  import { DockBarMode, WegPinnedItemsVisibility } from "@dockbar/lib/types";
  import { DragDropProvider, DragOverlay } from "@dnd-kit/svelte";
  import { move } from "@dnd-kit/helpers";
  import { onDestroy } from "svelte";
  import { BackgroundByLayers } from "libs/ui/svelte/components/BackgroundByLayers";
  import { t } from "../i18n/index.ts";
  import { dockState, listToGroups } from "../state/items.svelte.ts";
  import { settingsState } from "../state/settings.svelte.ts";
  import { systemState } from "../state/system.svelte.ts";
    import { dockShouldBeHidden, setDockIsDraggingItem } from "../state/hidden.svelte.ts";
  import { getWindowsForItem, windowsState } from "../state/windows.svelte.ts";
  import { getDockBarMenu } from "../dockMenu.ts";
  import { createDragDropManager, RestrictToBoundsDescriptor } from "libs/ui/dnd.ts";
  import { setupMagnification } from "../state/magnify.svelte.ts";
  import type { SwItem } from "../types.ts";
  import DockItemsGroup from "./DockItemsGroup.svelte";
  import DockItemSwitch from "./DockItemSwitch.svelte";

  // Workaround for https://github.com/clauderic/dnd-kit/issues/2112
  const manager = createDragDropManager({
    // keep the drag overlay inside the dock bar: the overlay would otherwise
    // follow the pointer into the empty strip below the bar and poke out from
    // behind its rounded corners as a white sliver at the screen bottom
    modifiers: [RestrictToBoundsDescriptor],
  });
  onDestroy(() => manager.destroy());

  let dockItemsEl: HTMLDivElement | null = $state(null);

  $effect(() => {
    if (dockItemsEl) {
      setupMagnification(dockItemsEl);
    }
  });

  const settings = $derived(settingsState.value as any);
  const isHorizontal = $derived(settings?.position === "Top" || settings?.position === "Bottom");

  function isItemVisible(item: SwItem): boolean {
    const pinnedVisibility = settings?.pinnedItemsVisibility as WegPinnedItemsVisibility;
    const monitor = systemState.currentMonitor;
    const showPinned = pinnedVisibility === WegPinnedItemsVisibility.Always || monitor.isPrimary;

    if (item.type !== "AppOrFile") {
      return showPinned;
    }
    if (item.pinned && showPinned) {
      return true;
    }
    // temporal items of running windows (placed after the right separator)
    return getWindowsForItem(item as any, windowsState.dockWindows).length > 0;
  }

  // splits the flat items array (left..., left-separator, center..., right-separator, ...right)
  // into their three groups, same as the toolbar does
  const groupedItems = $derived(listToGroups(dockState.items, true));
  const visibleGroupedItems = $derived.by(() => ({
    left: groupedItems.left.filter(isItemVisible),
    center: groupedItems.center.filter(isItemVisible),
    right: groupedItems.right.filter(isItemVisible),
  }));

  const isEmpty = $derived(
    [
      ...visibleGroupedItems.left,
      ...visibleGroupedItems.center,
      ...visibleGroupedItems.right,
    ].filter((c) => c.type !== "Separator").length === 0,
  );

  const itemIndexById = $derived.by(() => {
    const map = new Map<string, number>();
    dockState.items.forEach((item, i) => map.set(item.id, i));
    return map;
  });

  function onContextMenu(e: MouseEvent) {
    const alignX = settingsState.popupAlignX;
    const alignY = settingsState.popupAlignY;
    const cursor = { x: e.clientX, y: e.clientY };
    invoke(DockBarCommand.TriggerContextMenu, {
      menu: { ...getDockBarMenu($t, cursor), alignX, alignY },
      forwardTo: null,
    });
  }

  function handleDragStart() {
    setDockIsDraggingItem(true);
  }

  function handleDragEnd(event: any) {
    setDockIsDraggingItem(false);
    // Reorder once on drop, never during the drag: the items live in three
    // keyed each blocks (left/center/right), so an onDragOver reorder makes
    // Svelte tear down and recreate the dragged element's DOM across groups.
    // dnd-kit's placeholder and drop animation keep referencing the old node,
    // which manifests as a stray white sliver flashing under the dock bar.
    const newIds = move(
      dockState.items.map((item) => item.id),
      event,
    );
    dockState.items = newIds.map((id) => dockState.items.find((item) => item.id === id)!);
  }
</script>

<div
  role="toolbar"
  tabindex="0"
  data-has-margin={!!settings?.margin}
  data-size={settings?.mode === DockBarMode.FullWidth ? "full-width" : "min-content"}
  class="taskbar {settingsState.position.toLowerCase()}"
  class:horizontal={isHorizontal}
  class:vertical={!isHorizontal}
  class:hidden={dockShouldBeHidden.value}
  oncontextmenu={onContextMenu}
>
  <BackgroundByLayers />
  <div class="dock-items-container">
    <DragDropProvider
      {manager}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <div class="dock-items" bind:this={dockItemsEl}>
        {#if isEmpty}
          <span class="dock-empty-state-label">{$t("dock.empty")}</span>
        {:else}
          <DockItemsGroup id="left" items={visibleGroupedItems.left} {itemIndexById} />
          <DockItemsGroup id="center" items={visibleGroupedItems.center} {itemIndexById} />
          <DockItemsGroup id="right" items={visibleGroupedItems.right} {itemIndexById} />
        {/if}
      </div>

      <DragOverlay>
        {#snippet children(source)}
          {@const overlayItem = dockState.items.find((c) => c.id === source.id)}
          {#if overlayItem}
            <DockItemSwitch item={overlayItem} isOverlay />
          {/if}
        {/snippet}
      </DragOverlay>
    </DragDropProvider>
  </div>
</div>
