<div class="stats__label t-uppercase t-microlabel">
  <slot name="label" />
</div>
<output class="stats__value t-medium">
  <slot name="value" />
</output>

<style>
  .stats {
    &__label {
      color: theme(--color-text-3);
    }

    &__value {
      color: theme(--color-text-4);
    }
  }
</style>
