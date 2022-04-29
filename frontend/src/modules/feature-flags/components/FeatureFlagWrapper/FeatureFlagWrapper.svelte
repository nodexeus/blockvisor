<script lang="ts">
  export let shouldDisplay: boolean;
</script>

{#if shouldDisplay}
  <slot />
{:else}
  <slot name="fallback" />
{/if}
