<article class="block">
  <div class="block__title t-uppercase">
    <slot name="title" />
  </div>
  <div class="block__content">
    <slot />
  </div>
</article>

<style>
  .block {
    display: grid;
    gap: 28px;
    padding-top: 20px;
    margin-top: 40px;
    border-top: 1px solid theme(--color-text-5-o10);

    @media (--screen-large) {
      padding-top: 40px;
      margin-top: 80px;
      grid-template-columns: 296px auto;
    }

    &__content {
      color: theme(--color-text-4);
    }
  }
</style>
