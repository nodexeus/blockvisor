<script lang="ts">
  export let size = 'medium';
  export let style = 'primary';
  export let border = 'rounded';
  export let cssCustom = '';
  export let asLink = false;

  const rootClass = 'u-button-reset icon-button';
  const cssClass = [rootClass, border, size, style, cssCustom].join(' ');
</script>

{#if asLink}
  <a href={$$props.href} on:click class={cssClass} {...$$props}>
    <slot />
  </a>
{:else}
  <button class={cssClass} on:click {...$$props}>
    <slot />
  </button>
{/if}

<style lang="postcss">
  .icon-button {
    aspect-ratio: 1;
    display: inline-flex;
    font-weight: var(--font-weight-bold);
    justify-content: center;
    align-items: center;
    gap: 10px;
    @mixin font button;
    text-decoration: none;

    &[disabled] {
      cursor: not-allowed;
      opacity: 0.4;
    }
  }

  .rounded {
    border-radius: 4px;
  }

  .round {
    border-radius: 50%;
  }

  .normal {
    width: 56px;
  }

  .medium {
    width: 40px;
  }

  .small {
    width: 32px;
  }

  .tiny {
    width: 24px;
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
    color: theme(--color-text-2);
    border: 1px solid theme(--color-text-2);
    background-color: transparent;

    &:hover,
    &:active {
      color: theme(--color-border-3);
      border: 2px solid theme(--color-text-2);
    }
  }

  .ghost {
    color: theme(--color-text-2);
    transition: color 0.18s var(--transition-easing-cubic);

    &:hover,
    &:active,
    &:focus {
      color: theme(--color-border-3);
    }
  }
</style>
