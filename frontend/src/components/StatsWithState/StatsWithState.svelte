<script lang="ts">
  import anime from 'animejs';

  import { onMount } from 'svelte';

  export let state = 'default';
  export let value: number | string;
  export let id: string;
  export let changeOnZero = true;

  const isNumericalValue = typeof value === 'number';

  const parentClass = [
    'stats',
    `stats--${state}`,
    `stats--${changeOnZero ? 'change-on-zero' : 'static'}`,
    `stats--${
      isNumericalValue ? (value > 0 ? 'has-value' : 'zero-value') : 'nan'
    }`,
  ].join(' ');

  const initialValue = isNumericalValue ? 0 : value;

  onMount(() => {
    if (!isNumericalValue) {
      return;
    }

    anime({
      targets: `#${id} > .stats__value`,
      innerText: [0, value],
      delay: 100,
      duration: 800,
      round: true,
      easing: 'linear',
    });
  });
</script>

<div {id} class={parentClass}>
  <p class="t-xxxlarge stats__value">{initialValue}</p>
  <p class="stats__description t-microlabel"><slot /></p>
</div>

<style>
  .stats {
    color: theme(--color-text-4);

    & :global(svg) {
      flex-shrink: 0;
    }

    & :global(svg path) {
      fill: currentColor;
    }

    &--inactive {
      color: theme(--color-text-2);
    }

    &--active {
      color: theme(--color-primary);
    }

    &__description {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    &__value {
      margin-bottom: 8px;
    }

    &--change-on-zero {
      &:global(.stats--zero-value) {
        color: theme(--color-text-2);
      }
    }
  }
</style>
