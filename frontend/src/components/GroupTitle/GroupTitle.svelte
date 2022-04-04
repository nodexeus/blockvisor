<header class="group-title">
  <h3 class="t-medium group-title__heading">
    <slot name="title" />
  </h3>
  <slot name="action" />
  <div class="group-title__stats t-uppercase t-microlabel">
    <slot name="stats" />
  </div>
</header>

<style>
  .group-title {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 12px;
    justify-content: space-between;

    @media (--screen-medium-small) {
      justify-content: flex-start;
    }

    &__heading {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 65%;

      @media (--screen-medium-small) {
        max-width: 50%;
      }
    }

    &__stats {
      flex-basis: 100%;
      display: flex;
      align-items: center;
      gap: 8px;
      color: theme(--color-text-3);
    }
  }
</style>
