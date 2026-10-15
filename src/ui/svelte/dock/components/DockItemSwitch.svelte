<script lang="ts">
  import type { SwItem } from "../types.ts";
  import UserApplication from "./items/UserApplication.svelte";
  import Separator from "./items/Separator.svelte";

  interface Props {
    item: SwItem;
    isOverlay?: boolean;
  }

  let { item, isOverlay = false }: Props = $props();
</script>

{#if item.type === "AppOrFile"}
  <UserApplication {item} {isOverlay} />
{:else if item.type === "Separator"}
  <Separator {item} />
{/if}
