<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import { toast } from 'components/Toast/Toast';
  import { ENDPOINTS } from 'consts/endpoints';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';
  import { getUserInfo } from 'utils';
  import { httpClient } from 'utils/httpClient';
  import {
    getOrganisationsByUserId,
    selectedOrganization,
    getOrganisationById,
  } from '../store/organisationStore';

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

  function handleReset() {
    $form.orgName.reset({ value: $selectedOrganization.name });
  }
</script>

<div class="form-wrapper">
  <form use:form>
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
  <div class="form-buttons">
    <Button on:click={handleSubmit} size="small" style="secondary">Save</Button>
    <Button on:click={handleReset} style="ghost" size="medium" type="reset"
      >Discard Changes</Button
    >
  </div>
</div>

<style>
  .form-wrapper {
    margin-top: 60px;
    max-width: 400px;
  }

  .form-buttons {
    margin-top: 40px;
    display: flex;
    gap: 36px;
    align-items: center;
  }
</style>
