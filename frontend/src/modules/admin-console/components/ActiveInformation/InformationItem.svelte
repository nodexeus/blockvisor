<script lang="ts">
  export let items = [];
</script>

<article class="info-item">
  <div class="t-medium info-item__title">
    <slot name="title" />
  </div>

  <div class="info-item__info">
    {#if Boolean(items.length)}
      <table class="table info-item__table">
        {#each items as { title, info }}
          <tr class="table__row info-item__item">
            <th
              class="table__heading info-item__col t-microlabel t-uppercase info-item__item-title"
            >
              {title}
            </th>
            <td class="info-item__item-info info-item__col">{info}</td>
          </tr>
        {/each}
        <colgroup>
          <col width="180" />
          <col />
        </colgroup>
      </table>
    {/if}
    {#if $$slots.default}
      <div class="info-item__content">
        <slot />
      </div>
    {/if}
  </div>
</article>

<style>
  .info-item {
    display: flex;
    gap: 28px;
    flex-direction: column;

    @media (--screen-medium) {
      flex-direction: row;
    }

    @media (--screen-large) {
      gap: 28px 136px;
    }

    &__content {
      display: flex;
      gap: 12px;
      padding-top: 20px;
    }

    &__title {
      @media (--screen-medium) {
        flex: 0 1 188px;
      }
    }

    &__col {
      padding-top: 28px;
      padding-bottom: 8px;
    }

    &__info {
      flex-grow: 1;
    }

    &__item {
      &:first-child {
        & > :global(*) {
          padding-top: 0;
        }
      }
      &-title {
        color: theme(--color-text-3);
      }

      &-info {
        color: theme(--color-text-4);
      }
    }
  }
</style>
