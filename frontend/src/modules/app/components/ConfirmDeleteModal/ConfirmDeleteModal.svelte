<script lang="ts">
  import Button from 'components/Button/Button.svelte';

  import Modal from 'components/Modal/Modal.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { onMount } from 'svelte';
  import { valueMatch } from 'utils/valueMatch';

  export let targetValue = 'DELETE';
  export let isModalOpen = false;
  export let handleModalClose;
  export let id = 'js-confirm-delete';

  export let form;

  onMount(() => {
    const input = document.querySelector(`#${id} .input__field`) as HTMLElement;

    if (!input) {
      return;
    }

    input.focus();
  });
</script>

<Modal {id} on:submit {form} {handleModalClose} isActive={isModalOpen}>
  <svelte:fragment slot="header">Are You Sure?</svelte:fragment>
  <input
    type="hidden"
    style="display:none;"
    name="targetValue"
    value={targetValue}
    readonly
  />

  <Input
    validate={[valueMatch]}
    labelClass="t-small s-bottom--micro"
    name="confirm"
    field={$form?.confirm}
  >
    <slot name="label" slot="label" />
  </Input>

  <slot />

  <div slot="footer" class="t-right">
    <Button disabled={!$form.valid} size="small" style="primary">Confirm</Button
    >
  </div>
</Modal>
