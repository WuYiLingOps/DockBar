<script lang="ts">
  import type { SwItem } from "../types.ts";
  import DraggableItem from "./DraggableItem.svelte";
  import DockItemSwitch from "./DockItemSwitch.svelte";

  interface Props {
    id: string;
    items: SwItem[];
    itemIndexById: Map<string, number>;
  }

  let { id, items, itemIndexById }: Props = $props();
</script>

<div class="dock-items-{id}">
  {#each items as item (item.id)}
    <DraggableItem {item} index={itemIndexById.get(item.id) ?? 0}>
      <DockItemSwitch {item} />
    </DraggableItem>
  {/each}
</div>
