<script lang="ts">
  import { fly } from 'svelte/transition';
  import { flyDefault } from 'consts/animations';
  import CopyIcon from 'icons/copy-12.svg';
  import { copy } from 'svelte-copy';

  export let value = '';

  let disabled = false;

  const actionEnable = () => {
    disabled = false;
  };

  const handleCopy = () => {
    disabled = true;
    setTimeout(actionEnable, 800);
  };
</script>

<button
  {disabled}
  class="u-button-reset copy-node"
  on:svelte-copy={handleCopy}
  use:copy={value}
>
  {#if disabled}
    <span
      in:fly|local={flyDefault}
      out:fly|local={flyDefault}
      class="t-tiny copy-node__bubble">Copied</span
    >
  {/if}
  <slot />
  <CopyIcon />
</button>

<style>
  .copy-node {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: theme(--color-text-5-o10);

    & :global(svg) {
      transition: color 0.18s var(--transition-easing-cubic);
      min-width: 12px;
    }

    &:hover {
      color: theme(--color-text-3);
    }

    &:active {
      color: theme(--color-text-5);
    }

    &__bubble {
      position: absolute;
      background-color: theme(--color-utility-success);
      color: theme(--color-text-5);
      border-radius: 16px;
      padding: 2px 8px;
      right: -50%;
      top: 0;
      transform: translate3d(-50%, -150%, 0);
    }
  }
</style>
