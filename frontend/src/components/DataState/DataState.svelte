<script lang="ts">
  import StateIcon from 'components/StateIcon/StateIcon.svelte';

  export let status: HostState | NodeState;
</script>

<div class="data-state">
  <StateIcon {status} />
  {status}
</div>

<style>
  .data-state {
    display: inline-flex;
    align-items: center;
    gap: 8px;

    & :global(svg) {
      flex-basis: 12px;
      flex-shrink: 0;
    }
  }
</style>
