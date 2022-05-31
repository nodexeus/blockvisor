<script>
  import Button from 'components/Button/Button.svelte';

  import Modal from 'components/Modal/Modal.svelte';
  import TagsField from 'modules/forms/components/TagsField/TagsField.svelte';

  export let handleModalClose;
  export let isModalOpen;

  import { useForm } from 'svelte-use-form';
  import OrganisationUser from './OrganisationUser.svelte';

  const form = useForm();

  $: console.log($form.values.emails);
</script>

<Modal id="org-members" {handleModalClose} isActive={isModalOpen} size="large">
  <svelte:fragment slot="header">Manage "BlockJoy" Members</svelte:fragment>
  <form
    class="org-members__invite"
    use:form
    on:change={() => {
      console.log($form);
    }}
  >
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

  <OrganisationUser />
  <OrganisationUser />
  <OrganisationUser pending={true} />
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
