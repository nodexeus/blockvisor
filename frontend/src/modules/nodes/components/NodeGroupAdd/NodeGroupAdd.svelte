<script>
  import IconAdd from 'icons/plus-12.svg';
</script>

<button on:click class="u-button-reset node-group-add t-tiny" {...$$restProps}>
  <IconAdd />
  <slot>Add a Group</slot>
</button>

<style>
  .node-group-add {
    color: theme(--color-text-5);
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;

    &:not(:disabled):hover,
    &:active {
      background-color: theme(--color-text-5-o3);
    }

    &:disabled {
      cursor: not-allowed;
    }

    & :global(svg) {
      color: theme(--color-text-2);
    }
  }
</style>
