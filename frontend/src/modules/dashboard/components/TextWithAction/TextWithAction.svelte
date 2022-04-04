<div class="text-with-action">
  <div class="text-with-action__title">
    <slot name="title" />
  </div>
  <div class="text-with-action__action">
    <slot name="action" />
  </div>
  <p class="text-with-action__content">
    <slot />
  </p>
</div>

<style>
  .text-with-action {
    display: flex;
    flex-wrap: wrap;
    gap: 24px;
    border-bottom: 1px solid theme(--color-text-5-o10);
    padding-bottom: 24px;
    margin-bottom: 40px;
    color: var(--color-text-5);
    flex-direction: column;

    @media (--screen-medium-small) {
      flex-direction: row;
      align-items: center;
      justify-content: space-between;
    }

    &__content {
      flex-basis: 100%;
      color: var(--color-text-3);
    }
  }
</style>
