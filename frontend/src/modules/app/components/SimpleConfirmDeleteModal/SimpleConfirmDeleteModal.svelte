<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  function handleConfirm() {
    dispatch('confirm');
  }

  function handleCancel() {
    dispatch('cancel');
  }

  export let isModalOpen = false;
  export let handleModalClose;
  export let id = 'js-confirm-delete';
  export let loading;
</script>

<Modal {id} {handleModalClose} isActive={isModalOpen}>
  <svelte:fragment slot="header">Are You Sure?</svelte:fragment>
  <slot name="content" />
  <div slot="footer" class="t-right">
    {#if loading}
      <LoadingSpinner size="small" id="confirm-modal" />
    {:else}
      <Button handleClick={handleCancel} size="small" style="ghost"
        >Cancel</Button
      >
      <Button handleClick={handleConfirm} size="small" style="warning"
        >Delete</Button
      >
    {/if}
  </div>
</Modal>
