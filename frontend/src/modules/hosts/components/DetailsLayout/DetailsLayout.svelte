<script lang="ts">
  import { fade } from 'svelte/transition';
  import { pageTransition } from 'consts/animations';
</script>

<div class="host-details grid-spacing">
  <nav class="host-details__nav">
    <slot name="nav" />
  </nav>
  <section in:fade={pageTransition} class="host-details__content">
    <slot />
  </section>
</div>

<style>
  .host-details {
    grid-column: span 12;
    margin-top: 32px;
    overflow: hidden;
    width: 100%;

    @media (--screen-large) {
      grid-column: 2/11;
    }

    &__nav {
      margin-bottom: 32px;
    }

    &__content {
      position: relative;

      @media (--screen-medium-small) {
        padding-left: 36px;
      }
    }
  }
</style>
