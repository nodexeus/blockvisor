<script lang="ts">
  import type { InputUtilProps } from './InputUtil.svelte';

  export let position: InputUtilProps['position'] = 'left';

  const utilClasses = ['input-util', `input-util--${position}`].join(' ');
</script>

<span class={utilClasses}>
  <slot />
</span>

<style>
  .input-util {
    color: theme(--color-text-2);
    fill: theme(--color-text-2);
    position: absolute;
    top: 50%;
    transform: translate3d(0, -50%, 0);
  }

  .input-util--left {
    left: 12px;
  }

  .input-util--right {
    right: 12px;
  }

  .input-util :global(svg path) {
    fill: currentColor;
  }
</style>
