<script lang="ts">
  import { stepChange } from 'consts/animations';

  import { fly } from 'svelte/transition';
  export let title;

  export let size: string = 'medium';
</script>

<article
  on:outroend
  transition:fly|local={stepChange}
  class={`layout layout--${size}`}
>
  <h1 class="t-xxlarge-fluid layout__title">{title}</h1>

  <slot />
</article>

<style>
  .layout {
    grid-column: span 12;
    width: 100%;
    margin: 0 auto;
    max-width: 404px;

    @media (--screen-large) {
      margin: 0;
      max-width: none;
      grid-column: 5/9;
    }

    &--large {
      @media (--screen-medium) {
        grid-column: span 4;
      }
    }

    &--large {
      grid-column: span 12;
      width: 100%;
      margin: 0 auto;
      max-width: 512px;

      @media (--screen-large) {
        margin: 0;
        max-width: none;
        grid-column: 5/10;
      }

      &--large {
        @media (--screen-medium) {
          grid-column: span 4;
        }
      }
    }

    &__title {
      margin-top: 16px;
      margin-bottom: clamp(2.5rem, 7vw, 3.75rem);

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
