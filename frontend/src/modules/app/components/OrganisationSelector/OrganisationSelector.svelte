<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Dropdown from 'components/Dropdown/Dropdown.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import { toast } from 'components/Toast/Toast';
  import IconCaret from 'icons/caret-micro.svg';
  import IconAdd from 'icons/plus-12.svg';
  import CreateNewOrganisation from 'modules/organisation/components/CreateNewOrganisation.svelte';
  import {
    activeOrganisation,
    getOrganisations,
    organisations,
    setActiveOrganisation,
  } from 'modules/organisation/store/organisationStore';
  import { onMount } from 'svelte';
  import { clickOutside, getUserInfo } from 'utils';

  let dropdownActive: boolean = false;
  let createNewActive: boolean = false;

  function handleClickOutside() {
    dropdownActive = false;
  }

  function handleClickOutsideCreateNew() {
    createNewActive = false;
  }

  onMount(() => {
    getOrganisations(getUserInfo().id);
  });

  function handleSelectOrganisation(orgId: string) {
    setActiveOrganisation(orgId).then(() => {
      dropdownActive = false;

      toast.success(
        `Your active organisation is set to  ${$activeOrganisation.name}`,
      );
    });
  }
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
        >{$activeOrganisation
          ? $activeOrganisation?.name.substring(0, 1)?.toUpperCase()
          : ''}</span
      ></span
    >
    <span class="organisation-selector__name t-smaller t-normal"
      >{$activeOrganisation?.name || ''}</span
    >
    <IconCaret /></Button
  >

  {#if $organisations}
    <Dropdown isActive={dropdownActive}>
      <DropdownLinkList>
        <p
          class="t-uppercase t-micro t-color-text-4 organisation-selector__title"
        >
          Organisations
        </p>
        {#each $organisations as item}
          <li>
            <DropdownItem
              as="button"
              on:click={() => handleSelectOrganisation(item.id)}
              >{item.name || ''}</DropdownItem
            >
          </li>
        {/each}
        <li class="organisation-selector__new organisation-selector__divider">
          <DropdownItem
            on:click={() => {
              createNewActive = true;
            }}
            size="large"
            as="button"
            ><IconAdd />
            Add&nbsp;Organisation</DropdownItem
          >
        </li>
      </DropdownLinkList>
    </Dropdown>
  {/if}
  <CreateNewOrganisation
    isModalOpen={createNewActive}
    handleModalClose={handleClickOutsideCreateNew}
  />
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
  .organisation-selector__title {
    padding: 12px;
    letter-spacing: 1px;
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

  .organisation-selector__divider {
    margin-top: 8px;
    border-top: 1px solid theme(--color-text-5-o10);
  }
</style>
