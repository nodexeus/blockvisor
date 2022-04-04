<li class="user-details-item">
  <div class="user-details-item__label t-microlabel t-color-text-3 t-uppercase">
    <slot name="label" />
  </div>
  <div>
    <slot />
  </div>
</li>

<style>
  .user-details-item {
    padding: 28px 0 8px;
    border-bottom: 1px solid theme(--color-text-5-o10);
    display: flex;
    flex-direction: column;
    gap: 8px 28px;

    @media (--screen-large) {
      flex-direction: row;
      align-items: baseline;
    }

    &__label {
      min-width: 190px;
    }
  }
</style>
