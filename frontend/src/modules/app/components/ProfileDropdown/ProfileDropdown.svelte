<script lang="ts">
  import Dropdown from 'components/Dropdown/Dropdown.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import { clickOutside } from 'utils';
  import Avatar from 'components/Avatar/Avatar.svelte';

  import IconAccount from 'icons/person-12.svg';
  import IconDocument from 'icons/document-12.svg';
  import IconCog from 'icons/cog-12.svg';
  import IconDoor from 'icons/door-12.svg';
  import { ROUTES } from 'consts/routes';
  import { user } from 'modules/authentication/store';
  import axios from 'axios';

  let isActive = false;

  const handleClickOutside = () => {
    isActive = false;
  };

  const onDropdownToggle = () => {
    isActive = !isActive;
  };

  const activateDropdown = (e) => {
    if (!['Enter'].includes(e.key)) {
      return;
    }

    isActive = true;
  };

  const handleLogout = async () => {
    axios.post(ROUTES.AUTH_LOGOUT).then(() => {
      location.reload();
    });
  };

  $: fullName =
    $user && $user.firstName && $user.lastName
      ? `${$user?.firstName ?? ''} ${$user?.lastName ?? ''}`
      : 'John Doe';
  export let src = '';
</script>

<div
  class="profile-dropdown"
  use:clickOutside
  on:click_outside={handleClickOutside}
>
  <Avatar
    tabindex="0"
    on:click={onDropdownToggle}
    on:keyup={activateDropdown}
    {fullName}
    {src}
    size="medium-small"
  />
  <Dropdown {isActive}>
    <DropdownLinkList>
      <li>
        <DropdownItem href="#">
          <IconAccount />
          Profile</DropdownItem
        >
      </li>
      <li>
        <DropdownItem href="#">
          <IconDocument />
          Billing</DropdownItem
        >
      </li>
      <li>
        <DropdownItem href={ROUTES.PROFILE_SETTINGS}>
          <IconCog />
          Settings</DropdownItem
        >
      </li>
      <li class="profile-dropdown__item--with-divider">
        <DropdownItem as="button" on:click={handleLogout}>
          <IconDoor />

          Logout</DropdownItem
        >
      </li>
    </DropdownLinkList>
  </Dropdown>
</div>

<style>
  .profile-dropdown {
    position: relative;

    &__item--with-divider {
      border-top: 1px solid theme(--color-text-5-o10);
    }

    & :global(.dropdown) {
      right: 0;
      min-width: 160px;
      margin-top: 8px;
    }
  }
</style>
