<div class="select">
  <div>
    <slot name="title" />

    <small class="t-small select__description">
      <slot name="description" />
    </small>
  </div>

  <slot name="action" />
</div>

<style>
  .select {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding-top: 24px;
    padding-bottom: 10px;
    align-items: flex-start;
    border-bottom: 1px solid theme(--color-text-5-o10);

    @media (--screen-medium-small) {
      flex-direction: row;
      justify-content: space-between;
    }

    &__description {
      margin-top: 8px;
      display: block;
      color: theme(--color-text-2);
    }
  }
</style>
