<script lang="ts">
  export let title: string = '';
</script>

<hr class="form-section__divider" />

<h2 class="form-section__title t-large">
  {title}
</h2>
<div class="form-section__content">
  <slot />
</div>

<style>
  .form-section {
    &__divider {
      grid-column: 2/11;
      border-width: 0;
      border-bottom: 1px solid theme(--color-text-5-o10);
      grid-column: span 12;

      @media (--screen-medium-small) {
        grid-column: 2/11;
      }

      @media (--screen-large) {
        grid-column: 2/8;
      }
    }
    &__title {
      grid-column: span 12;

      @media (--screen-medium-small) {
        padding-top: 40px;
        padding-bottom: 60px;
      }

      @media (--screen-medium-small) {
        grid-column: 2/5;
      }

      @media (--screen-large) {
        grid-column: 2/4;
      }
    }
    &__content {
      grid-column: span 12;

      @media (--screen-medium-small) {
        padding-top: 40px;
        padding-bottom: 60px;
      }

      @media (--screen-medium-small) {
        grid-column: 6/11;
      }

      @media (--screen-large) {
        grid-column: 5/8;
      }

      & :global(li:not(:last-child)) {
        margin-bottom: 24px;
      }

      & :global(li:last-child) {
        margin-top: 40px;
      }
    }
  }
</style>
