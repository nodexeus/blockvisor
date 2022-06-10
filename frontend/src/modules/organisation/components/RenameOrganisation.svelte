<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import { toast } from 'components/Toast/Toast';
  import { ENDPOINTS } from 'consts/endpoints';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { onMount } from 'svelte';
  import { Hint, required, useForm } from 'svelte-use-form';
  import { getUserInfo } from 'utils';
  import { httpClient } from 'utils/httpClient';
  import { getOrganisations } from '../store/organisationStore';

  const form = useForm();
  export let handleModalClose: VoidFunction;
  export let isModalOpen: boolean = false;
  export let organisationName: string;
  export let organisationId: string;

  onMount(() => {
    $form.orgName.value = organisationName;
  });

  async function handleSubmit() {
    if (!$form.values.orgName) {
      toast.warning('Organisation name is cannot be empty.');
      return;
    }

    try {
      const res = await httpClient.put(
        ENDPOINTS.ORGANISATIONS.UPDATE_ORGANISATION(organisationId),
        {
          name: $form.values.orgName,
        },
      );

      if (res.status === 200) {
        handleModalClose();
        getOrganisations(getUserInfo().id);
        toast.success('Organisation renamed successfully.');
      }
    } catch (error) {
      toast.warning('Something went wrong');
    }
  }
</script>

<Modal id="new-org" {handleModalClose} isActive={isModalOpen} size="large">
  <svelte:fragment slot="header">Rename {organisationName}</svelte:fragment>

  <form use:form on:submit|preventDefault={handleSubmit}>
    <Input
      size="large"
      validate={[required]}
      name="orgName"
      value={$form.orgName?.value}
      placeholder="BlockJoy"
      field={$form?.orgName}
    >
      <svelte:fragment slot="label">Organisation Name</svelte:fragment>
      <svelte:fragment slot="hints">
        <Hint on="required">This is a mandatory field</Hint>
      </svelte:fragment>
    </Input>
  </form>
  <div slot="footer">
    <Button on:click={handleSubmit} size="small" style="primary">Update</Button>
  </div>
</Modal>
