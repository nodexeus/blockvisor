<script lang="ts">
  import anime from 'animejs';

  import Logo from 'components/Logo/Logo.svelte';
  import { onMount } from 'svelte';

  export let title = '';

  const animateEntry = () =>
    anime({
      targets: `#js-auth-layout`,
      opacity: [0, 1],
      translateY: [8, 0],
      easing: 'easeInOutQuad',
      duration: 400,
    });

  onMount(animateEntry);
</script>

<main tabindex="0" id="content" class="layout">
  <section class="layout__wrapper" id="js-auth-layout">
    <header>
      <Logo />
      <h1 class="layout__title t-medium">{title}</h1>
    </header>
    <slot />
  </section>
</main>

<style lang="postcss">
  .layout {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 24px 12px;

    &__wrapper {
      opacity: 0;
      background-color: theme(--color-text-5-o3);
      border-radius: 8px;
      padding: 30px;

      @media (--screen-medium-small) {
        padding: 60px;
        width: 380px;
      }
    }

    &__title {
      margin-top: 40px;
      margin-bottom: 24px;
    }
  }
</style>
