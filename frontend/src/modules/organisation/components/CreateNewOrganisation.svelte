<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import { toast } from 'components/Toast/Toast';
  import { ENDPOINTS } from 'consts/endpoints';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';
  import { httpClient } from 'utils/httpClient';

  const form = useForm();
  export let handleModalClose;
  export let isModalOpen;
  export let loading: boolean = false;
  export let loadingCreate: boolean = false;

  async function handleSubmit() {
    if (!$form.values.orgName) {
      toast.warning('Organisation name is required');
      return;
    }

    try {
      const res = await httpClient.post(
        ENDPOINTS.ORGANISATIONS.CREATE_ORGANISATION_POST,
        {
          name: $form.values.orgName,
        },
      );

      if (res.status === 200) {
        handleModalClose();
        toast.success('Organisation created successfully');
      }
    } catch (error) {
      toast.warning('Something went wrong');
    }
  }
</script>

<Modal id="new-org" {handleModalClose} isActive={isModalOpen} size="large">
  <svelte:fragment slot="header">Create new organisation</svelte:fragment>

  <form use:form on:submit|preventDefault={handleSubmit}>
    <Input
      size="large"
      validate={[required]}
      name="orgName"
      placeholder="BlockJoy"
      field={$form?.orgName}
    >
      <svelte:fragment slot="label">Organisation Name</svelte:fragment>
      <svelte:fragment slot="hints">
        <Hint on="required">This is a mandatory field</Hint>
      </svelte:fragment>
    </Input>
    <Button size="medium" style="secondary">Create</Button>
  </form>
</Modal>
