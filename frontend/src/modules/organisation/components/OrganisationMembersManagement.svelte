<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { ENDPOINTS } from 'consts/endpoints';
  import TagsField from 'modules/forms/components/TagsField/TagsField.svelte';
  import { onMount } from 'svelte';
  import { useForm } from 'svelte-use-form';
  import { httpClient } from 'utils/httpClient';
  import type { OrgUser } from '../models/OrgUser';
  import OrganisationUser from './OrganisationUser.svelte';

  const form = useForm();
  export let handleModalClose;
  export let isModalOpen;
  export let loading: boolean = false;
  export let organisationId: string;
  export let organisationName: string;
  let organisationUsers: OrgUser[] = [];

  onMount(async () => {
    loading = true;
    try {
      const res = await httpClient.get(
        ENDPOINTS.ORGANISATIONS.LIST_ORGANISATION_MEMBERS_GET(organisationId),
      );

      if (res.status === 200) {
        organisationUsers = res.data;
      }
      loading = false;
    } catch (error) {
      loading = false;
    }
  });
</script>

<Modal id="org-members" {handleModalClose} isActive={isModalOpen} size="large">
  <svelte:fragment slot="header"
    >Manage {organisationName} Members</svelte:fragment
  >
  {#if loading}
    <LoadingSpinner id="blockjoy" size="small" />
  {:else}
    <form class="org-members__invite" use:form on:change={() => {}}>
      <div class="org-members__input">
        <TagsField
          field={$form?.emails}
          name="emails"
          size="medium"
          limit={10}
          multiline
          showFull
          placeholder="Enter one or more e-mails"
        >
          <svelte:fragment slot="label">Invite Members</svelte:fragment>
        </TagsField>
      </div>
      <Button size="medium" style="secondary">Send&nbsp;Invite</Button>
    </form>

    {#each organisationUsers as item}
      <OrganisationUser {item} />
    {/each}
  {/if}
</Modal>

<style>
  .org-members__invite {
    margin-bottom: 12px;
  }
  .org-members__input {
    width: 100%;
    margin-bottom: 12px;
  }
</style>
