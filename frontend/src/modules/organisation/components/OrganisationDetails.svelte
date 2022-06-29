<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import { toast } from 'components/Toast/Toast';
  import { ENDPOINTS } from 'consts/endpoints';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { afterUpdate, onMount } from 'svelte';
  import { Hint, required, useForm , validators} from 'svelte-use-form';
  import { getUserInfo } from 'utils';
  import { httpClient } from 'utils/httpClient';
  import {
    getOrganisationsByUserId,
    selectedOrganization,
    getOrganisationById,
  } from '../store/organisationStore';

  /*  export let organisationName: string;
  export let organisationId: string; */

  const form = useForm();

  async function handleSubmit() {
    if (!$form.values.orgName) {
      toast.warning('Organisation name cannot be empty.');
      return;
    }

    try {
      const res = await httpClient.put(
        ENDPOINTS.ORGANISATIONS.UPDATE_ORGANISATION($selectedOrganization.id),
        {
          name: $form.values.orgName,
        },
      );

      if (res.status === 200) {
        getOrganisationsByUserId(getUserInfo().id);
        getOrganisationById($selectedOrganization.id);
        toast.success('Organisation created successfully');
      }
    } catch (error) {
      toast.warning('Something went wrong');
    }
  }
</script>

<div>
  <form use:form>
    <!-- <input type="text" name="orgName" value={$selectedOrganization.name} use:validators={[required]} /> -->
    <Input
      size="large"
      validate={[required]}
      name="orgName"
      value={$selectedOrganization.name}
      placeholder="BlockJoy"
    >
      <svelte:fragment slot="label">Organisation Name</svelte:fragment>
      <svelte:fragment slot="hints">
        <Hint on="required">This is a mandatory field</Hint>
      </svelte:fragment>
    </Input>
  </form>
  <Button on:click={handleSubmit} size="small" style="primary">Update</Button>
</div>
