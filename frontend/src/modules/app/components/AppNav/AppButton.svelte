<script lang="ts">
  export let activeApp;
  export let value;
</script>

<button
  on:click
  class="u-button-reset app-button"
  class:app-button--active={parseInt(value) === activeApp}
  {value}
  {...$$restProps}
>
  <slot />
</button>

<style>
  .app-button {
    background-color: transparent;
    border-radius: 10px;
    padding: 8px;
    color: theme(--color-text-5-o30);
    transition: background-color 0.2s var(--transition-easing-cubic),
      color 0.25s var(--transition-easing-cubic);

    &:not(&--active) {
      &:hover,
      &:active {
        color: theme(--color-text-4);
      }
    }

    &:hover,
    &:active,
    &:focus {
      & :global(.app-nav__label) {
        display: block;
      }
    }

    &--active {
      background-color: theme(--color-text-5-o10);
      color: theme(--color-primary);
    }
  }
</style>
