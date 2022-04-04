<div class="text-with-action-top">
  <div class="text-with-action-top__title">
    <slot name="title" />

    <p class="text-with-action-top__content">
      <slot />
    </p>
  </div>
  <div class="text-with-action-top__action">
    <slot name="action" />
  </div>
</div>

<style>
  .text-with-action-top {
    display: flex;
    gap: 24px;
    padding-bottom: 40px;
    margin-bottom: 40px;
    color: var(--color-text-5);
    flex-direction: column;

    @media (--screen-medium-small) {
      flex-direction: row;
      align-items: flex-start;
      justify-content: space-between;
    }

    @media (--screen-large) {
      gap: 83px;
    }

    &__content {
      margin-top: 20px;
      color: var(--color-text-3);
      max-width: 410px;
    }

    &__action {
      flex-shrink: 0;
    }
  }
</style>
