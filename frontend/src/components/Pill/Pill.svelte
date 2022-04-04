<script lang="ts">
  import { fly } from 'svelte/transition';

  import IconClose from 'icons/close-12.svg';

  export let removable = true;
  export let transition = { y: 12, duration: 180 };
</script>

<span in:fly|local={transition} class="pill t-small">
  <span class="pill__text" class:pill__text--with-action={removable}>
    <slot />
  </span>
  {#if removable}
    <button
      class="u-button-reset pill__button"
      type="button"
      on:click
      {...$$restProps}
    >
      <span class="visually-hidden">Remove</span>
      <IconClose aria-hidden="true" />
    </button>
  {/if}
</span>

<style>
  .pill {
    max-width: 20ch;
    color: theme(--color-text-4);
    background-color: theme(--color-text-5-o10);
    border-radius: 4px;
    padding: 4px 16px;
    display: inline-flex;
    align-items: center;
    gap: 8px;

    &__text {
      text-overflow: ellipsis;
      overflow: hidden;
      white-space: nowrap;
      max-width: 100%;

      &--with-action {
        max-width: calc(100% - 20px);
      }
    }

    &__button {
      flex-basis: 12px;
      color: theme(--color-border-4);
      opacity: 0.3;

      & :global(path) {
        transition: opacity 0.18s var(--transition-easing-cubic);
      }

      &:hover,
      &:active {
        opacity: 1;
      }
    }
  }
</style>
