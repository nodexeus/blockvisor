<script lang="ts">
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import NavItem from 'components/NavItem/NavItem.svelte';
  import IconDelete from 'icons/close-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import IconFolder from 'icons/folder-12.svg';
  import IconList from 'icons/list-12.svg';
  import IconEdit from 'icons/pencil-2-12.svg';
  import IconPerson from 'icons/person-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';
  import GroupEdit from 'modules/forms/components/GroupEdit/GroupEdit.svelte';

  export let isParent;
  export let href;
  export let id;
  export let title;
  export let isEditing;
  export let handleConfirm;
  export let handleEdit;
</script>

<li
  class:hierarchy-list__item--parent={isParent}
  class:node--parent={isParent}
  class:node--child={!isParent}
  class="hierarchy-list__item node"
>
  {#if isEditing}
    <GroupEdit {handleConfirm} value={title} />
  {:else}
    <NavItem on:click checkIfParent {href}>
      <svelte:fragment slot="icon">
        {#if isParent}
          <IconList />
        {:else}
          <IconFolder />
        {/if}
      </svelte:fragment>
      <svelte:fragment slot="label">{title}</svelte:fragment></NavItem
    >
    <slot />
    {#if !isParent}
      <div class="node__controls">
        <ButtonWithDropdown
          position="right"
          slot="action"
          iconButton
          buttonProps={{ style: 'ghost', size: 'small' }}
        >
          <svelte:fragment slot="label">
            <span class="visually-hidden">Open action dropdown</span>
            <span class="user-data-row__action-icon" aria-hidden="true">
              <IconDots />
            </span>
          </svelte:fragment>
          <DropdownLinkList slot="content">
            <li>
              <DropdownItem value={id} as="button" on:click={handleEdit}>
                <IconEdit />
                Rename</DropdownItem
              >
            </li>
            <li>
              <DropdownItem value={id} href="#">
                <IconPerson />
                Permissions</DropdownItem
              >
            </li>
            <li class="node__item-divider">
              <DropdownItem value={id} href="#">
                <IconDelete />
                Delete</DropdownItem
              >
            </li>
          </DropdownLinkList>
        </ButtonWithDropdown>
      </div>
    {/if}
  {/if}
</li>

<style>
  .node {
    position: relative;

    &__item-divider {
      border-top: 1px solid theme(--color-text-5-o10);
    }

    &--child {
      &:hover,
      &:focus-within {
        .node__controls {
          opacity: 1;
        }
      }
    }

    &__controls {
      transition: opacity 0.15s var(--transition-easing-cubic);
      opacity: 0;
      position: absolute;
      right: 0;
      top: 0;

      @media (--screen-medium-max) {
        opacity: 1;
      }
    }
  }
</style>
