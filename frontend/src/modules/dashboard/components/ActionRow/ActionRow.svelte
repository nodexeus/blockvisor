<div class="action-row">
  <div>
    <h3 class="t-large-fluid">
      <slot name="title" />
    </h3>
    <p class="t-small action-row__description">
      <slot name="description" />
    </p>
  </div>

  <div class="action-row__action">
    <slot name="action" />
  </div>
</div>

<style>
  .action-row {
    display: flex;
    margin: 40px 0;
    gap: 24px;
    flex-direction: column;

    @media (--screen-medium-small) {
      gap: 100px;
      justify-content: space-between;
      align-items: center;
      flex-direction: row;
    }

    &__description {
      margin-top: 20px;
      color: theme(--color-text-3);
    }

    &__action {
      min-width: 120px;
    }
  }
</style>
