<script lang="ts">
  import DataRow from 'components/DataRow/DataRow.svelte';
  import TokenIcon from 'components/TokenIcon/TokenIcon.svelte';
  import { ROUTES } from 'consts/routes';

  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';

  import IconAccount from 'icons/person-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconDots from 'icons/dots-12.svg';

  import CopyNode from 'modules/dashboard/components/CopyNode/CopyNode.svelte';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';

  export let data;
</script>

{#each data as { icon, state, name, node, ip, added } (node)}
  <DataRow {state}>
    <TokenIcon slot="icon" {icon} />
    <svelte:fragment slot="primary">
      <a class="u-link-reset" href={ROUTES.NODE_DETAILS(node)}>{name}</a>
      <CopyNode value={node}>
        <small>{node}</small>
      </CopyNode>
    </svelte:fragment>

    <svelte:fragment slot="secondary">
      <span>{ip}</span>
      <span>Added {added} ago</span>
    </svelte:fragment>

    <ButtonWithDropdown
      iconButton
      position="right"
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
