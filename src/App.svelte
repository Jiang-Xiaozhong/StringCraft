<script lang="ts">
  import FloatBar from "./components/FloatBar.svelte";
  import Settings from "./components/Settings.svelte";

  let { windowLabel }: { windowLabel: string } = $props();
</script>

{#if windowLabel === "settings"}
  <Settings />
{:else}
  <FloatBar />
{/if}
