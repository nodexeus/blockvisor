<li class="host-form-data">
  <div class="host-form-data__label t-uppercase t-label">
    <slot name="label" />:
  </div>
  <output class="host-form-data__value t-large">
    <slot name="value" />
  </output>
</li>

<style>
  .host-form-data {
    color: theme(--color-text-3);
    display: grid;
    align-items: center;
    grid-gap: 16px;
    padding-bottom: 24px;
    margin-bottom: 24px;
    border-bottom: 1px solid theme(--color-text-5-o10);

    @media (--screen-medium-small) {
      grid-template-columns: 194px auto;
      grid-gap: 24px;
    }

    &__value {
      color: theme(--color-text-5);
    }
  }
</style>
