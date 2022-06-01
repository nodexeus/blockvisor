<script>
  import Pill from 'components/Pill/Pill.svelte';
  import IconPerson from 'icons/person-12.svg';
</script>

<ul class="u-list-reset filter-group">
  <li>
    <Pill removable={false}>[Type]</Pill>
  </li>
  <li>
    <Pill removable={false} disabled>[is]</Pill>
  </li>
  <li>
    <Pill removable>[Something]</Pill>
  </li>
</ul>

<style>
  .filter-group {
    display: flex;
    gap: 2px;
  }
</style>
