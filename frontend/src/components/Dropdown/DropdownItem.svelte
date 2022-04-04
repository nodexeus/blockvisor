<script lang="ts">
  export let as: 'a' | 'button' = 'a';
</script>

{#if as === 'a'}
  <a {...$$restProps} class="t-tiny link dropdown-item"><slot /></a>
{:else}
  <button on:click {...$$restProps} class="t-tiny u-button-reset dropdown-item">
    <slot />
  </button>
{/if}

<style>
  .dropdown-item {
    color: theme(--color-text-4);
    width: 100%;
    display: flex;
    justify-content: flex-start;
    gap: 12px;
    align-items: center;
    padding: 8px 12px;
    transition: background-color 0.18s var(--transition-easing-cubic);

    &,
    &:visited {
      color: theme(--color-text-4);
    }

    & :global(svg) {
      color: theme(--color-text-5-o20);
      flex-basis: 12px;
      transition: color 0.18s var(--transition-easing-cubic);
      pointer-events: none;
    }

    &:hover,
    &:active,
    &:focus {
      text-decoration: none;
      background-color: theme(--color-text-5-o3);

      & :global(svg) {
        color: theme(--color-text-5);
      }
    }
  }
</style>
