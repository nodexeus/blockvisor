<tr class="table__row service-row">
  <td data-label="Service Name" class="service-row__col"
    >Monthly Subscription</td
  >
  <td data-label="Quantity" class="service-row__col">1</td>
  <td data-label="Price" class="service-row__col">$100</td>
  <td data-label="Total" class="service-row__col">$100</td>
</tr>

<style>
  .service-row {
    @media (--screen-large-max) {
      display: block;
      margin-bottom: 12px;
    }

    &__col {
      padding: 28px 20px 12px 0;

      @media (--screen-large-max) {
        padding: 0 0 12px;
        display: flex;
        align-items: baseline;
        gap: 12px;

        &::before {
          content: attr(data-label) ':';
          text-transform: uppercase;
          color: theme(--color-text-4);
          @mixin font microlabel;
        }
      }

      &:last-child {
        padding-right: 0;
      }
    }
  }
</style>
