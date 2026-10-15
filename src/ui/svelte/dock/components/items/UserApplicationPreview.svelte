<script lang="ts">
  import { invoke, DockBarCommand } from "@dockbar/lib";
  import { Icon, MissingIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { windowsState, previews } from "../../state/windows.svelte.ts";

  interface Props {
    title: string;
    hwnd: number;
  }

  let { title, hwnd }: Props = $props();

  const preview = $derived(previews.value[hwnd]);

  function onClick() {
    invoke(DockBarCommand.WegToggleWindowState, {
      hwnd,
      wasFocused: windowsState.delayedFocused?.hwnd === hwnd,
    });
  }

  function onAuxClick(e: MouseEvent) {
    if (e.button === 1) {
      invoke(DockBarCommand.WegCloseApp, { hwnd });
    }
  }

  function onClose(e: MouseEvent) {
    e.stopPropagation();
    invoke(DockBarCommand.WegCloseApp, { hwnd });
  }
</script>

<div
  role="button"
  tabindex="0"
  class="dock-item-preview"
  onclick={onClick}
  onauxclick={onAuxClick}
  onkeypress={() => {}}
>
  <div class="dock-item-preview-topbar">
    <div class="dock-item-preview-title">{title}</div>
    <button data-skin="transparent" onclick={onClose}>
      <Icon iconName="IoClose" />
    </button>
  </div>
  <div class="dock-item-preview-image-container">
    {#if preview}
      <img class="dock-item-preview-image" src="data:image/webp;base64,{preview.data}" alt={title} />
    {:else}
      <MissingIcon class="dock-item-no-preview" />
    {/if}
  </div>
</div>
