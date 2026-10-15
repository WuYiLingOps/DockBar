<script lang="ts">
  import { invoke, DockBarCommand } from "@dockbar/lib";
  import { DockBarSide, WegMiddleClickAction, WindowAttention, type UserAppWindow } from "@dockbar/lib/types";
  import { FileIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { t } from "../../i18n/index.ts";
  import type { AppOrFileDockItem } from "../../types.ts";
  import { settingsState } from "../../state/settings.svelte.ts";
  import { windowsState, focused } from "../../state/windows.svelte.ts";
  import { getUserApplicationContextMenu, launchItem } from "../../appMenu.ts";
  import { triggerPreviewWidget } from "../../previewWidget.ts";
  import { CssHandled } from "libs/ui/svelte/utils/animations.ts";
  import { onDestroy } from "svelte";
  import { getBadge } from "../../state/badge.svelte.ts";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";

  interface Props {
    item: AppOrFileDockItem;
    windows: UserAppWindow[];
  }

  let { item, windows }: Props = $props();

  const settings = $derived(settingsState.value as any);

  const badge = $derived(getBadge(item, windows));

  const itemLabel = $derived(
    settings?.showWindowTitle && windows.length ? windows[0]!.title : null,
  );
  const isFocused = $derived(windows.some((w) => w.hwnd === focused.value?.hwnd));
  const isFlashing = $derived(windows.some((w) => w.attention === WindowAttention.Flashing));
  const needsAttention = $derived(
    isFlashing || windows.some((w) => w.attention === WindowAttention.Highlighted),
  );

  let itemEl: HTMLDivElement | null = $state(null);

  let tooltipVisible = $state(false);
  let tooltipX = $state(0);
  let tooltipY = $state(0);
  let tooltipTimeout: ReturnType<typeof setTimeout> | undefined = undefined;

  function clearTooltipTimeout() {
    if (tooltipTimeout !== undefined) {
      clearTimeout(tooltipTimeout);
      tooltipTimeout = undefined;
    }
  }

  function placeTooltip() {
    if (!itemEl) {
      return;
    }
    // position: fixed coordinates (viewport-relative), immune to ancestor
    // overflow clipping, which would cut the tooltip outside the dock.
    const rect = itemEl.getBoundingClientRect();
    const position = settingsState.position;
    const gap = 12;
    const x = rect.left + rect.width / 2;
    const y =
      position === DockBarSide.Top
        ? rect.bottom + gap
        : position === DockBarSide.Bottom
          ? rect.top - gap
          : rect.top + rect.height / 2;
    tooltipX = x;
    tooltipY = y;
  }

  function onItemPointerEnter() {
    if (!settings?.hoverShowAppName) {
      return;
    }
    clearTooltipTimeout();
    tooltipTimeout = setTimeout(() => {
      placeTooltip();
      tooltipVisible = true;
    }, 150);
  }

  function onItemPointerLeave() {
    clearTooltipTimeout();
    tooltipVisible = false;
  }

  onDestroy(clearTooltipTimeout);

  function onClick() {
    if (windows.length > 1) {
      triggerPreviewWidget(itemEl!, windows);
      return;
    }

    const win = windows[0];
    if (!win) {
      launchItem(item, false);
    } else {
      invoke(DockBarCommand.WegToggleWindowState, {
        hwnd: win.hwnd,
        wasFocused: windowsState.delayedFocused?.hwnd === win.hwnd,
      });
    }
  }

  function onAuxClick(e: MouseEvent) {
    if (e.button !== 1) return;
    if (settings?.middleClickAction === WegMiddleClickAction.OpenNewInstance) {
      launchItem(item, false);
    } else {
      const win = windows[0];
      if (win) invoke(DockBarCommand.WegCloseApp, { hwnd: win.hwnd });
    }
  }

  function onContextMenu(e: MouseEvent) {
    e.stopPropagation();
    const alignX = settingsState.popupAlignX;
    const alignY = settingsState.popupAlignY;
    invoke(DockBarCommand.TriggerContextMenu, {
      menu: { ...getUserApplicationContextMenu($t, item, windows), alignX, alignY },
      forwardTo: null,
    });
  }
</script>

<div bind:this={itemEl} role="menu" tabindex="0" class="dock-item-overlay">
  <div
    role="menuitem"
    tabindex="0"
    class="dock-item"
    onclick={onClick}
    onauxclick={onAuxClick}
    oncontextmenu={onContextMenu}
    onpointerenter={onItemPointerEnter}
    onpointerleave={onItemPointerLeave}
    onkeypress={() => {}}
  >
    <FileIcon class="dock-item-icon" path={item.relaunch?.icon || item.path} umid={item.umid} />
    {#if itemLabel}
      <div class="dock-item-title">{itemLabel}</div>
    {/if}

    {#if needsAttention}
      <div
        class="dock-item-attention-indicator"
        class:dock-item-attention-indicator-flashing={isFlashing}
      ></div>
    {/if}
  </div>

  {#if tooltipVisible && item.displayName}
    <div
      class="dock-item-tooltip"
      class:dock-item-tooltip-below={settingsState.position === DockBarSide.Top}
      style:left="{tooltipX}px"
      style:top="{tooltipY}px"
      transition:CssHandled
    >{item.displayName}</div>
  {/if}

  {#if badge}
    <div class="dock-item-badge" transition:CssHandled>
      {#if badge.type === "image"}
        <img src={badge.src} alt="" />
      {/if}
    </div>
  {/if}

  {#if settings?.showInstanceCounter && windows.length > 1}
    <div class="dock-item-instance-counter-badge" transition:CssHandled>
      {windows.length}
    </div>
  {/if}

  {#if !settings?.showWindowTitle}
    <div
      class="dock-item-open-sign"
      class:dock-item-open-sign-active={windows.length > 0}
      class:dock-item-open-sign-focused={isFocused}
      class:dock-item-open-sign-attention={needsAttention}
    ></div>
  {/if}
</div>

<style>
  .dock-item-badge-color {
    width: 100%;
    height: 100%;
  }
</style>
