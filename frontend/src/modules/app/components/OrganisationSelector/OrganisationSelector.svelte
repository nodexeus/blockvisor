<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Dropdown from 'components/Dropdown/Dropdown.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import IconCaret from 'icons/caret-micro.svg';
  import {
    activeOrganisation,
    getOrganisations,
    organisations,
  } from 'modules/organisation/store/organisationStore';
  import { onMount } from 'svelte';
  import { clickOutside, getUserInfo } from 'utils';

  let dropdownActive: boolean = false;

  function handleClickOutside() {
    dropdownActive = false;
  }

  onMount(() => {
    getOrganisations(getUserInfo().id);
  });
</script>

<div
  class="organisation-selector"
  use:clickOutside
  on:click_outside={handleClickOutside}
>
  <Button
    on:click={() => (dropdownActive = !dropdownActive)}
    size="small"
    style="basic"
  >
    <span class="organisation-selector__letter"
      ><span class="t-tiny"
        >{$activeOrganisation?.name.substring(0, 1)?.toUpperCase()}</span
      ></span
    >
    <span class="organisation-selector__name t-smaller t-normal"
      >{$activeOrganisation?.name || ''}</span
    >
    <IconCaret /></Button
  >

  <Dropdown isActive={dropdownActive}>
    <DropdownLinkList>
      {#each $organisations as item}
        <li>
          <DropdownItem href="#">{item.name || ''}</DropdownItem>
        </li>
      {/each}
    </DropdownLinkList>
  </Dropdown>
</div>

<style>
  .organisation-selector {
    position: relative;

    & :global(.dropdown) {
      left: 0;
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
