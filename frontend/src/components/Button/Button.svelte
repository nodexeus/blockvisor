<script lang="ts">
  import type { ButtonProps } from './Button.svelte';

  export let size: ButtonProps['size'] = 'normal';
  export let style: ButtonProps['style'] = 'primary';
  export let border: ButtonProps['border'] = 'rounded';
  export let display: ButtonProps['display'] = 'inline';
  export let cssCustom: ButtonProps['cssCustom'] = '';
  export let asLink = false;
  export let handleClick;

  const rootClass = 'u-button-reset button';
  $: cssClass = [rootClass, border, size, display, style, cssCustom].join(' ');
</script>

{#if asLink}
  <a href={$$props.href} class={cssClass} {...$$props}>
    <slot />
  </a>
{:else}
  <button class={cssClass} on:click|preventDefault={handleClick}>
    <slot />
  </button>
{/if}

<style lang="postcss">
  .button {
    font-weight: var(--font-weight-bold);
    justify-content: center;
    align-items: center;
    gap: 10px;
    @mixin font button;
    text-decoration: none;
    transition: background-color 0.18s var(--transition-easing-cubic),
      box-shadow 0.18s var(--transition-easing-cubic),
      color 0.18s var(--transition-easing-cubic),
      border-color 0.18s var(--transition-easing-cubic);

    &[disabled] {
      cursor: not-allowed;
      opacity: 0.4;
    }
  }

  .inline {
    display: inline-flex;
  }

  .block {
    display: flex;
    width: 100%;
  }

  .rounded {
    border-radius: 4px;
  }

  .round {
    border-radius: 50%;
  }

  .normal {
    padding: 16px 24px;
  }

  .medium {
    padding: 8px 28px;
  }

  .small {
    padding: 6px 16px;
    @mixin font button-small;
  }

  .primary {
    background-color: theme(--color-primary);
    color: theme(--color-foreground-secondary);
    box-shadow: 0px 0px 0px 3px var(--color-primary-o0);
    transition: box-shadow 0.18s var(--transition-easing-cubic);

    &:hover,
    &:active {
      box-shadow: 0px 0px 0px 3px var(--color-primary-o30);
    }
  }

  .secondary {
    background-color: theme(--color-secondary);
    color: theme(--color-foreground-secondary);
    box-shadow: 0px 0px 0px 3px var(--color-secondary-o0);
    transition: box-shadow 0.18s var(--transition-easing-cubic);

    &:hover,
    &:active {
      box-shadow: 0px 0px 0px 3px var(--color-secondary-o30);
    }
  }

  .outline {
    color: theme(--color-text-5);
    border: 1px solid theme(--color-border-2);
    background-color: transparent;
    transition: background-color 0.18s var(--transition-easing-cubic);

    &:hover,
    &:active {
      background-color: theme(--color-text-5-o10);
    }
  }

  .light {
    border: 1px solid var(--color-foreground-secondary-o10);
    background-color: transparent;

    &:hover,
    &:active,
    &:focus {
      background-color: var(--color-foreground-secondary-o10);
    }
  }

  .light-o10 {
    border: 1px solid var(--color-text-5-o10);
    background-color: transparent;

    &:hover,
    &:active,
    &:focus {
      background-color: var(--color-text-5-o10);
    }
  }

  .round-medium {
    width: 40px;
    height: 40px;
  }

  .ghost {
    color: theme(--color-text-5);
    transition: background-color 0.18s var(--transition-easing-cubic);

    &:hover,
    &:active,
    &:focus {
      background-color: var(--color-text-5-o3);
    }
  }

  .warning {
    background-color: theme(--color-utility-warning);
    color: theme(--color-foreground-primary);
    box-shadow: 0px 0px 0px 3px var(--color-utility-warning-o0);
    transition: box-shadow 0.18s var(--transition-easing-cubic);

    &:hover,
    &:active {
      box-shadow: 0px 0px 0px 3px var(--color-primary-o30);
    }
  }

  .basic {
    color: theme(--color-text-5);
    padding: 0;
  }
</style>
