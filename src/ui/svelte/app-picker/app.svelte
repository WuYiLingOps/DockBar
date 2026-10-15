<script lang="ts">
  import { Widget } from "@dockbar/lib";
  import { t, locale } from "./i18n/index.ts";
  import { settings, appPickerState, isAlreadyOnDock, pinApp, closePicker } from "./state.svelte.ts";
  import FileIcon from "libs/ui/svelte/components/Icon/FileIcon.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  $effect(() => {
    locale.set(settings.value.language);
  });

  let searchInput: HTMLInputElement;

  $effect(() => {
    // reset the filter and focus the box each time the picker is shown
    appPickerState.search = "";
    searchInput?.focus();
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      closePicker();
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="slu-std-popover app-picker" onkeydown={onkeydown}>
  <!-- drag region: hold and move to reposition the picker window -->
  <div class="drag-bar" data-tauri-drag-region>
    <span class="picker-title">{$t("title")}</span>
  </div>

  {#if appPickerState.loading}
    <div class="picker-empty">{$t("empty_loading")}</div>
  {:else if appPickerState.filtered.length === 0}
    <div class="picker-empty">{$t("empty")}</div>
  {:else}
    <!-- the grid is the main body: all installed apps, the search box only filters -->
    <div class="apps-grid">
      {#each appPickerState.filtered as app (app.umid ?? app.path.toString())}
        <button
          data-skin="default"
          class="app-entry"
          class:pinned={isAlreadyOnDock(app)}
          title={app.displayName}
          onclick={() => pinApp(app)}
        >
          <FileIcon class="app-icon" path={app.path.toString() || null} umid={app.umid} />
          <span class="app-name">{app.displayName}</span>
        </button>
      {/each}
    </div>
  {/if}

  <input
    bind:this={searchInput}
    bind:value={appPickerState.search}
    class="search-box"
    type="text"
    data-skin="default"
    placeholder={$t("search_placeholder")}
    spellcheck="false"
  />
</div>

<style>
  .app-picker {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-s);
    width: 460px;
    height: 420px;
  }

  .drag-bar {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    height: 28px;
    /* net over the popover padding so the drag area reaches the window edges */
    margin: calc(-1 * var(--spacing-xs));
    padding: 0 var(--spacing-xs);
    border-radius: var(--radius-m) var(--radius-m) 0 0;
    cursor: grab;
    user-select: none;
  }

  .drag-bar:active {
    cursor: grabbing;
  }

  .picker-title {
    font-weight: 600;
  }

  .apps-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(68px, 1fr));
    gap: var(--spacing-2xs);
    flex: 1;
    align-content: start;
    overflow-y: auto;
    overscroll-behavior: contain;
  }

  .picker-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
  }

  .app-entry {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-2xs);
    padding: var(--spacing-2xs);
    border: none;
    border-radius: var(--radius-m);
    background: transparent;
    cursor: pointer;
    text-align: center;
  }

  .app-entry:hover {
    background: var(--dockbar-std-ui-hover-color);
  }

  .app-entry.pinned {
    outline: 1px solid var(--system-accent-color);
  }

  .app-icon {
    width: 22px;
    height: 22px;
  }

  .app-name {
    max-width: 100%;
    font-size: 0.7rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    pointer-events: none;
  }

  .search-box {
    flex-shrink: 0;
  }
</style>
