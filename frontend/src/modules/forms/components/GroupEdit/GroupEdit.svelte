<script lang="ts">
  import IconFolder from 'icons/folder-12.svg';
  import { onMount } from 'svelte';
  import { clickOutside } from 'utils';

  export let handleConfirm;
  export let value = '';
  let inputElement;

  const handleChange = (e) => {
    if (!['Enter'].includes(e.key)) {
      return;
    }

    handleConfirm();
  };

  onMount(() => {
    inputElement.focus();
  });
</script>

<div class="new-group">
  <span class="new-group__icon">
    <IconFolder />
  </span>
  <label for="new-group" class="visually-hidden">Add new group</label>
  <input
    bind:value
    use:clickOutside
    on:keydown={handleChange}
    on:click_outside={handleConfirm}
    bind:this={inputElement}
    id="new-group"
    type="text"
    class="new-group__input"
    placeholder="Group name"
  />
</div>

<style>
  .new-group {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    align-items: baseline;

    &__input {
      @mixin font tiny;
      background: transparent;
      padding: 0;
      color: theme(--color-text-5);
      border-width: 0;
      outline: 0;
    }

    &__icon {
      color: theme(--color-secondary);
      flex-basis: 12px;
    }
  }
</style>
