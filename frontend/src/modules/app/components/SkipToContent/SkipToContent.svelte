<script lang="ts">
  export let target = 'content';

  const focusOnContent = (e) => {
    e.preventDefault();
    e.stopPropagation();
    document.getElementById(target)?.focus({ preventScroll: true });
  };
</script>

<!-- svelte-ignore a11y-invalid-attribute -->
<a
  href="#"
  on:click={focusOnContent}
  rel="external"
  class="visually-hidden skip-to-content"
  {...$$restProps}
>
  <slot>Skip to main content.</slot>
</a>

<style>
  .skip-to-content {
    &:focus {
      all: initial;
      cursor: pointer;
      background-color: theme(--color-text-1);
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      text-align: center;
      padding: 20px;
      color: theme(--color-primary);
      @mixin font base;
      font-family: theme(--font-family-primary);
      border-bottom: 1px solid theme(--color-text-5-o10);
      z-index: var(--level-4);
    }
  }
</style>
