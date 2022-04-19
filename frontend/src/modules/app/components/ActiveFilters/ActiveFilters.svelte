<script>
  import FilterGroup from './FilterGroup.svelte';
</script>

<ul class="u-list-reset active-filters">
  <li>
    <FilterGroup />
  </li>
  <li>
    <FilterGroup />
  </li>
</ul>

<style>
  .active-filters {
    display: flex;
    overflow-x: auto;
    overflow-y: hidden;
    gap: 12px;
    padding-top: 20px;
    margin-top: 20px;
    border-top: 1px solid theme(--color-text-5-o10);
  }
</style>
