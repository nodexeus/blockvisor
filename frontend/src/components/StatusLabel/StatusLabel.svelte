<script lang="ts">
  export let status = 'default';

  $: classes = ['t-small status-label', `status-label--${status}`].join(' ');
</script>

<div class={classes}>
  <slot />
</div>

<style>
  .status-label {
    display: inline-flex;
    padding: 4px 16px;
    align-items: center;
    justify-content: center;
    color: theme(--color-text-4);
    border-radius: 4px;
    border: 1px solid theme(--color-text-5-o10);
  }
</style>
