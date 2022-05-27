<script>
  import Modal from 'components/Modal/Modal.svelte';
  import TagsField from 'modules/forms/components/TagsField/TagsField.svelte';

  export let handleModalClose;
  export let isModalOpen = true;

  import { useForm } from 'svelte-use-form';
  import OrganisationUser from './OrganisationUser.svelte';

  const form = useForm();

  $: console.log($form.values.emails);
</script>

<Modal id="org-members" {handleModalClose} isActive={isModalOpen}>
  <svelte:fragment slot="header">Manage "BlockJoy" Members</svelte:fragment>
  <form
    class="org-members__invite"
    use:form
    on:change={() => {
      console.log($form);
    }}
  >
    <TagsField
      field={$form?.emails}
      name="emails"
      size="medium"
      limit={10}
      multiline
    >
      <svelte:fragment slot="label">Invite Members</svelte:fragment>
    </TagsField>
  </form>

  <OrganisationUser />
  <OrganisationUser />
  <OrganisationUser pending={true} />
</Modal>

<style>
  .org-members__invite {
    margin-bottom: 12px;
  }
</style>
