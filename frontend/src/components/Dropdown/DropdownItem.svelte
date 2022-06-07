<script lang="ts">
  export let as: 'a' | 'button' = 'a';
  export let size: 'large' | 'small' = 'small';
</script>

{#if as === 'a'}
  <!-- svelte-ignore a11y-missing-attribute -->
  <a
    {...$$restProps}
    class={`t-tiny link dropdown-item ${
      size === 'large' ? 'dropdown-item--large' : 'dropdown-item--small'
    }`}><slot /></a
  >
{:else}
  <button
    on:click
    {...$$restProps}
    class={`t-tiny u-button-reset dropdown-item ${
      size === 'large' ? 'dropdown-item--large' : 'dropdown-item--small'
    }`}
  >
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

    transition: background-color 0.18s var(--transition-easing-cubic);
    cursor: pointer;

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

  .dropdown-item--large {
    padding: 16px 12px;
  }

  .dropdown-item--small {
    padding: 8px 12px;
  }
</style>
