<script>
  import { fade } from 'svelte/transition';
  import { browser } from '$app/env';
  import Button from 'components/Button/Button.svelte';
  import { ROUTES } from 'consts/routes';
  import Portal from 'svelte-portal/src/Portal.svelte';
</script>

{#if browser}
  <Portal hidden={true} target="#content">
    <aside out:fade={{ delay: 200, duration: 200 }} class="fallback">
      <div class="fallback__wrapper">
        <h2 class="t-xlarge-fluid s-bottom--large">
          BlockVisor is Coming Soon
        </h2>
        <p class="t-small s-bottom--large fallback__content">
          BlockVisor functionalities are currently under development. Until then
          you can use the Broadcasts feature.
        </p>
        <Button style="secondary" size="small" asLink href={ROUTES.BROADCASTS}
          >Go to Broadcasts</Button
        >
      </div>
    </aside>
  </Portal>
{/if}

<style>
  .fallback {
    isolation: isolate;
    position: fixed;
    top: 58px;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: theme(--color-overlay-background-2-o50);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    transform: translate3d(300px, 0, 0);
    padding: 20px;

    @media (--screen-smaller) {
      transform: translate3d(350px, 0, 0);
    }

    @media (--screen-medium-large) {
      transform: none;
      top: 65px;
      left: 60px;
    }

    &__wrapper {
      transform: translate3d(0, -50%, 0);
      background-color: theme(--color-overlay-background-1);
      padding: 40px;
      max-width: 404px;
    }

    &__content {
      color: theme(--color-text-3);
    }
  }
</style>
