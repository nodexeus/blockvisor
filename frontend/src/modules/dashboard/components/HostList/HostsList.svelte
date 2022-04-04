<script lang="ts">
  import DataRow from 'components/DataRow/DataRow.svelte';

  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconAccount from 'icons/person-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconDots from 'icons/dots-12.svg';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';

  export let data = [];
</script>

{#each data as { name, ip, location, state, url } (ip)}
  <DataRow {state}>
    <svelte:fragment slot="primary">
      <a href={url} class="u-link-reset">{name}</a>
    </svelte:fragment>

    <svelte:fragment slot="secondary">
      <span>{ip}</span>
      <address>{location}</address>
    </svelte:fragment>
    <ButtonWithDropdown
      position="right"
      iconButton
      buttonProps={{ style: 'ghost', size: 'tiny' }}
      slot="action"
    >
      <svelte:fragment slot="label">
        <IconDots />
      </svelte:fragment>
      <DropdownLinkList slot="content">
        <li>
          <DropdownItem as="button" href="#">
            <IconAccount />
            Profile</DropdownItem
          >
        </li>
        <li>
          <DropdownItem as="button" href="#">
            <IconDocument />
            Billing</DropdownItem
          >
        </li>
      </DropdownLinkList>
    </ButtonWithDropdown>
  </DataRow>
{/each}
