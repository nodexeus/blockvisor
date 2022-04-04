<tr class="table__row host-data-row">
  <th
    class="table__heading t-uppercase t-microlabel host-data-row__col  host-data-row__col--heading"
  >
    <slot name="title" />
  </th>

  <td class="host-data-row__col">
    <slot />
  </td>
</tr>

<style>
  .host-data-row {
    @media (--screen-smaller-max) {
      display: block;
    }

    &__col {
      color: theme(--color-text-4);
      padding: 28px 28px 8px 0;

      @media (--screen-smaller-max) {
        display: block;
        padding: 0 0 12px 0;
      }

      &:last-child {
        padding-right: 0;
      }

      &--heading {
        color: theme(--color-text-3);

        @media (--screen-smaller-max) {
          padding: 32px 0 12px;
        }
      }
    }
  }
</style>
