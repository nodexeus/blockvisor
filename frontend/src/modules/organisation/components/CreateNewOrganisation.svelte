<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';

  const form = useForm();
  export let handleModalClose;
  export let isModalOpen;
  export let loading: boolean = false;
</script>

<Modal id="new-org" {handleModalClose} isActive={isModalOpen} size="large">
  <svelte:fragment slot="header">Create new organisation</svelte:fragment>

  <form use:form on:change={() => {}}>
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
