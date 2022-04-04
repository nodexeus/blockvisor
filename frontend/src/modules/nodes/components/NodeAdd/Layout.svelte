<script lang="ts">
  import { stepChange } from 'consts/animations';

  import { fly } from 'svelte/transition';
  export let title;

  export let size: string = 'medium';
</script>

<section
  on:outroend
  transition:fly|local={stepChange}
  class={`layout layout--${size}`}
>
  <h1 class="t-xxlarge-fluid layout__title">{title}</h1>

  <slot />
</section>

<style>
  .layout {
    grid-column: span 12;
    width: 100%;
    margin: 0 auto;

    @media (--screen-medium-large) {
      max-width: none;
      grid-column: 4/12;
    }

    @media (--screen-large) {
      margin: 0;
      grid-column: 5/12;
    }

    &--large {
      @media (--screen-medium) {
        grid-column: span 6;
      }
    }

    &--large {
      grid-column: span 12;
      width: 100%;
      margin: 0 auto;
      max-width: none;

      @media (--screen-large) {
        max-width: 512px;
        margin: 0;
        max-width: none;
        grid-column: 5/11;
      }

      &--large {
        @media (--screen-medium) {
          grid-column: span 6;
        }
      }
    }

    &__title {
      margin-top: 16px;
      margin-bottom: 28px;

      @media (--screen-large) {
        margin-top: 40px;
      }
    }

    :global(.step__input-item) {
      margin-bottom: 24px;
    }

    :global(.step__inputs) {
      margin-bottom: 40px;
    }
  }
</style>
