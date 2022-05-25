<script lang="ts">
  import Button from 'components/Button/Button.svelte';

  import Dropdown from 'components/Dropdown/Dropdown.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import IconCaret from 'icons/caret-micro.svg';

  let dropdownActive: boolean = false;
</script>

<div class="organisation-selector">
  <Button
    on:click={() => (dropdownActive = !dropdownActive)}
    size="small"
    style="basic"
  >
    <span class="organisation-selector__letter"
      ><span class="t-tiny">B</span></span
    >
    <span class="organisation-selector__name t-smaller t-normal"
      >Acme Corporation</span
    >
    <IconCaret /></Button
  >

  <Dropdown isActive={dropdownActive}>
    <DropdownLinkList>
      <li>
        <DropdownItem href="#">jedan</DropdownItem>
      </li>
      <li>
        <DropdownItem href="#">jedan</DropdownItem>
      </li>
      <li>
        <DropdownItem href="#">jedan</DropdownItem>
      </li>
    </DropdownLinkList>
  </Dropdown>
</div>

<style>
  .organisation-selector {
    position: relative;

    & :global(.dropdown) {
      right: 0;
      min-width: 160px;
      margin-top: 8px;
    }
  }

  .organisation-selector__letter {
    display: flex;
    justify-content: center;
    align-items: center;
    background: var(--color-primary);
    width: 24px;
    height: 24px;
    border-radius: 50%;
    color: var(--color-text-1);
    font-weight: var(--font-weight-normal);
  }
  .organisation-selector__name {
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
