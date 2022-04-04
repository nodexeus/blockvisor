<script lang="ts">
  import { fade } from 'svelte/transition';
  import SpinnerGraphics from 'graphics/spinner.svg';
  import anime from 'animejs';
  import { onMount } from 'svelte';
  import { fadeDefault } from 'consts/animations';

  export let id = 'js-spinner';
  export let size = 'medium';

  const spinnerClass = ['spinner', `spinner--${size}`].join(' ');

  onMount(() => {
    anime({
      targets: `#${id} .spinner__element`,
      rotateZ: ['0deg', '90deg'],
      strokeDashoffset: [anime.setDashoffset, -135],
      loop: true,
      easing: 'easeInOutQuad',
      duration: 1000,
    });
  });
</script>

<aside class={spinnerClass} transition:fade={fadeDefault} {id}>
  <span class="visually-hidden">Loading</span>
  <SpinnerGraphics />
</aside>

<style>
  .spinner {
    & :global(svg) {
      aspect-ratio: 1;
    }

    &--button {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;
      justify-content: center;
      background-color: var(--color-text-1-o70);

      & :global(svg) {
        margin: 0 auto;
        width: 24px;
      }
    }

    &--medium {
      & :global(svg) {
        width: 60px;
      }
    }

    &--page {
      min-height: 100vh;
      min-width: 100vw;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;

      & :global(svg) {
        width: 100px;
      }
    }
  }

  .spinner :global(.spinner__element) {
    transform-origin: 50% 50%;
  }
</style>
