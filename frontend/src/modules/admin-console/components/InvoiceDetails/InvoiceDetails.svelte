<script>
  import InvoiceTotal from './InvoiceTotal.svelte';
  import ServiceRow from './ServiceRow.svelte';
</script>

<article
  class="invoice-details__info invoice-details__info--main container--medium-large"
>
  <div class="t-uppercase t-microlabel invoice-details__info-title">
    Invoice Recipient
  </div>
  <address class="invoice-details__info-address">
    Mark Mryant<br />
    Grounded Skynet LTD.<br />
    8625 South Drive<br />
    Buffalo, NY 14224<br />
    USA<br />
    VAT: 12345678912345<br />
  </address>
</article>

<article class="invoice-details invoice-details__info container--medium-large">
  <div class="t-uppercase t-microlabel invoice-details__info-title">
    Services
  </div>
  <div class="invoice-details__data">
    <table class="table invoice-details__table">
      <colgroup>
        <col />
        <col width="100" />
        <col width="200" />
        <col />
      </colgroup>
      <thead class="table__head">
        <tr class="t-uppercase t-color-text-4 t-microlabel">
          <th class="table__heading">Service name</th>
          <th class="table__heading">Quantity</th>
          <th class="table__heading">Price</th>
          <th class="table__heading">Total</th>
        </tr>
      </thead>
      <tbody class="t-small">
        <ServiceRow />
        <ServiceRow />
      </tbody>
      <tfoot>
        <InvoiceTotal />
      </tfoot>
    </table>
  </div>
</article>

<style>
  .invoice-details {
    &__data {
      flex-grow: 1;
    }

    &__table {
      @media (--screen-large-max) {
        &,
        & tbody {
          display: block;
        }

        & thead {
          display: none;
        }
      }
    }

    &__info {
      margin-bottom: 40px;
      padding-bottom: 40px;
      display: flex;
      flex-direction: column;
      gap: 12px 28px;

      @media (--screen-medium-small) {
        flex-direction: row;
      }

      &--main {
        border-bottom: 1px solid theme(--color-text-5-o10);
      }

      &-address {
        flex-grow: 1;
      }

      &-title {
        @media (--screen-medium-small) {
          flex: 0 0 188px;
        }
        color: theme(--color-text-3);
      }
    }
  }
</style>
