<script>
  import { fade } from 'svelte/transition';
  import { pageTransition } from 'consts/animations';
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import GroupTitle from 'components/GroupTitle/GroupTitle.svelte';
  import IconUser from 'icons/person-12.svg';
  import HeaderControls from 'modules/admin-console/components/HeaderControls/HeaderControls.svelte';
  import UserTable from 'modules/admin-console/components/UserTable/UserTable.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import Button from 'components/Button/Button.svelte';
  import Textarea from 'modules/forms/components/Textarea/Textarea.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';
  import { browser } from '$app/env';
  import { ROUTES } from 'consts/routes';
  import { onMount } from 'svelte';
  import { app } from 'modules/app/store';

  const form = useForm();

  let isModalOpen = false;

  const modalOpen = () => (isModalOpen = true);
  const modalClose = () => (isModalOpen = false);

  onMount(() => {
    setTimeout(() => {
      app.setBreadcrumbs([
        {
          title: 'Admin Console',
          url: ROUTES.ADMIN_CONSOLE_DASHBOARD,
        },
        {
          title: 'User Management',
          url: ROUTES.ADMIN_CONSOLE_USER_MANAGEMENT,
        },
      ]);
    }, 200);
  });
</script>

<section in:fade|local={pageTransition}>
  <ActionTitleHeader className="container--pull-back">
    <Button on:click={modalOpen} style="primary" size="small">Invite</Button>
    <h2 class="t-large" slot="title">User Management</h2>
  </ActionTitleHeader>
</section>

<section in:fade|local={pageTransition} class="container--medium-large">
  <GroupTitle>
    <svelte:fragment slot="title">Users</svelte:fragment>
    <svelte:fragment slot="stats">
      <IconUser />
      12 users in 5 groups
    </svelte:fragment>
  </GroupTitle>

  <HeaderControls />

  <UserTable />
</section>

{#if browser}
  <Modal
    {form}
    on:submit={(e) => {
      e.preventDefault();
      modalClose();
    }}
    handleModalClose={modalClose}
    isActive={isModalOpen}
  >
    <svelte:fragment slot="header">Invite New Members</svelte:fragment>
    <Textarea
      size="medium"
      field={$form?.emails}
      validate={[required]}
      name="emails"
      labelClass="visually-hidden"
      placeholder="Add one or multiple e-mails separated with comma."
    >
      <svelte:fragment slot="label">Member e-mails</svelte:fragment>
      <svelte:fragment slot="hints">
        <Hint on="required">This is a mandatory field</Hint>
      </svelte:fragment>
    </Textarea>
    <div class="t-right" slot="footer">
      <Button type="submit" style="primary" size="small">Confirm</Button>
    </div>
  </Modal>
{/if}
