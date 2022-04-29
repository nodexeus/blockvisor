<script>
  import Pill from 'components/Pill/Pill.svelte';
  import IconPerson from 'icons/person-12.svg';
</script>

<ul class="u-list-reset filter-group">
  <li>
    <Pill removable={false} transition={null}>
      <IconPerson slot="icon" />
      [Type]
    </Pill>
  </li>
  <li>
    <Pill removable={false} disabled transition={null}>[is]</Pill>
  </li>
  <li>
    <Pill removable transition={null}>[Something]</Pill>
  </li>
</ul>

<style>
  .filter-group {
    display: flex;
    gap: 2px;
  }
</style>
