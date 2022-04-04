<script lang="ts">
  import IconMenu from 'icons/menu-32.svg';

  export let menuClass = 'menu';
  export let handleClick;
</script>

<button
  on:click={handleClick}
  tabindex="-1"
  class={`u-button-reset ${menuClass}`}
>
  <IconMenu />
</button>

<style>
  .menu {
    display: block;
    padding: 6px 0 0 0;

    & :global(svg > *) {
      transform-origin: 50% 50%;
      transition: opacity 0.55s ease-out, transform 0.55s ease-out,
        stroke 0.2s var(--transition-easing-cubic);
    }

    &--active {
      & :global(.icon--middle) {
        transform: translate3d(-100%, 0, 0);
        opacity: 0;
      }

      & :global(.icon--top) {
        transform: rotate3d(0, 0, 1, -45deg) translate3d(0, 6px, 0);
      }

      & :global(.icon--bottom) {
        transform: rotate3d(0, 0, 1, 45deg) translate3d(0, -5px, 0);
      }
    }
  }
</style>
