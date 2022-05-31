<script lang="ts">
  import Portal from 'svelte-portal/src/Portal.svelte';
  import { focusTrap } from 'svelte-focus-trap';
  import { fade, fly } from 'svelte/transition';
  import { clickOutside } from 'utils';

  import IconClose from 'icons/close-12.svg';
  import { onDestroy, onMount } from 'svelte';
  import IconButton from 'components/IconButton/IconButton.svelte';

  export let form = null;
  export let size: 'normal' | 'large' = 'normal';
  export let isActive = false;
  export let handleModalClose;
  export let id = 'js-modal';

  const handleKeyDown = (e) => {
    if (e.key === 'Escape') {
      handleModalClose(e);
    }
  };

  onMount(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', handleKeyDown);
  });

  const handleCloseClick = (e) => {
    e.preventDefault();
    handleModalClose(e);
  };
</script>

<Portal hidden={true} target="body">
  {#if isActive}
    <aside
      {id}
      transition:fade={{ duration: 150 }}
      use:focusTrap
      class="modal-wrapper"
    >
      <section
        in:fly={{ y: 16, duration: 300, delay: 100 }}
        class={`modal ${size === 'large' ? 'modal--large' : 'modal--normal'}`}
        use:clickOutside
        on:click_outside={handleCloseClick}
      >
        <header class="modal__header">
          <div class="t-ellipsis">
            <slot name="header" />
          </div>
          <IconButton on:click={handleModalClose} style="ghost" size="tiny">
            <span class="visually-hidden">Close modal</span>
            <IconClose />
          </IconButton>
        </header>

        {#if form}
          <form use:form={form} on:submit {...$$restProps}>
            <article class="modal__content">
              <slot />
            </article>
            <footer class="modal__footer">
              <slot name="footer" />
            </footer>
          </form>
        {:else}
          <article class="modal__content">
            <slot />
          </article>
          <footer class="modal__footer">
            <slot name="footer" />
          </footer>
        {/if}
      </section>
    </aside>
  {/if}
</Portal>

<style>
  .modal {
    background-color: theme(--color-overlay-background-2);
    border-radius: 8px;
    width: 100%;

    @media (--screen-medium-large) {
      max-width: 540px;
    }
  }

  .modal--large {
    @media (--screen-medium-large) {
      max-width: 750px;
    }
  }

  .modal-wrapper {
    padding: 20px;
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: theme(--color-shadow-o80);
    z-index: var(--level-4);
  }

  .modal__header {
    display: flex;
    justify-content: space-between;
    gap: 40px;
    padding: 20px 28px;
    border-bottom: 1px solid theme(--color-text-5-o10);
    font-weight: bold;

    & :global(.button) {
      min-width: 44px;
    }
  }

  .modal__footer {
    padding: 20px 28px;
  }

  .modal__content {
    padding: 32px 28px 12px;
    max-height: calc(100vh - 200px);
    overflow: auto;
  }
</style>
