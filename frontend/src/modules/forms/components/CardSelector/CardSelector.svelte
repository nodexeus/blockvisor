<script lang="ts">
  import { fade } from 'svelte/transition';

  export let disabled = false;
  export let index: number;

  let isFocused = false;

  const handleFocus = () => {
    if (disabled) return;
    isFocused = true;
  };

  const handleBlur = () => {
    if (disabled) return;
    isFocused = false;
  };

  $: className = [
    'card-selector',
    `card-selector--${isFocused && !disabled ? 'focus' : 'blur'}`,
    `card-selector--${disabled ? 'disabled' : 'enabled'}`,
  ].join(' ');
</script>

<article
  in:fade={{ duration: 250, delay: index * 100 }}
  class={className}
  aria-checked={isFocused}
  on:focus={handleFocus}
  on:mouseenter={handleFocus}
  on:blur={handleBlur}
  on:mouseleave={handleBlur}
  role="radio"
>
  {#if disabled}
    <div class="card-selector__top">
      <span class="t-tiny">Coming soon</span>
    </div>
  {/if}
  <div class="card-selector__label">
    <slot name="label" />
  </div>
  {#if $$slots.action}
    <div class="card-selector__action">
      <slot name="action" />
    </div>
  {/if}
</article>

<style>
  .card-selector {
    background-color: var(--color-text-5-o3);
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    padding: 20px;
    aspect-ratio: 1;
    border-radius: 4px;
    border: 1px solid var(--color-primary-o0);
    transition: color 0.18s var(--transition-easing-cubic),
      border-color 0.18s var(--transition-easing-cubic);

    & :global(svg) {
      transition: color 0.18s var(--transition-easing-cubic);
    }

    &__label {
      color: theme(--color-text-3);
    }

    &__action {
      flex-grow: 1;
      display: flex;
      flex-direction: column;
      justify-content: flex-end;
      backface-visibility: hidden;
      transform: translateZ(0);
      will-change: max-height, opacity;
      transition: max-height 0.5s ease-out,
        opacity 0.25s var(--transition-easing-cubic);

      @media (--screen-medium-large) and (hover: hover) {
        max-height: 0px;
        opacity: 0;
      }
    }

    &--enabled {
      &:focus-within,
      &.card-selector--focus {
        color: var(--color-primary);
        border-color: var(--color-primary);

        & :global(svg) {
          color: var(--color-primary);
        }

        & .card-selector__action {
          transition: max-height 0.5s ease-in,
            opacity 0.25s var(--transition-easing-cubic);
          opacity: 1;
          max-height: 800px;
        }
      }
    }

    &--disabled {
      cursor: not-allowed;
      color: theme(--color-text-2);
      border: 1px solid theme(--color-border-2);
      background-color: transparent;
    }

    &__top {
      flex-grow: 1;
    }
  }
</style>
