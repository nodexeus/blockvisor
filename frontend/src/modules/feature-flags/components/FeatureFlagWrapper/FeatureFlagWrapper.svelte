<script lang="ts">
  export let shouldDisplay;
</script>

{#if shouldDisplay}
  <slot />
{:else}
  <slot name="fallback" />
{/if}
