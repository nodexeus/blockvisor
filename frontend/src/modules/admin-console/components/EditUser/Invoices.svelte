<script lang="ts">
  import Sorter from 'components/Sorter/Sorter.svelte';
  import InvoiceDataRow from './InvoiceDataRow.svelte';

  let sortActive;

  const handleSort = (id: string, value: SorterValues) => {
    sortActive = { id, value };
  };
</script>

<table class="table invoices">
  <colgroup>
    <col width="128" />
    <col />
    <col width="68" />
  </colgroup>
  <thead>
    <tr>
      <th class="table__heading">
        <Sorter active={sortActive} callback={handleSort} id="invoice-number"
          >Invoice #</Sorter
        >
      </th>
      <th class="table__heading">
        <Sorter active={sortActive} callback={handleSort} id="invoice-date"
          >Invoice Date</Sorter
        >
      </th>
      <th>
        <span class="visually-hidden">Actions</span>
      </th>
    </tr>
  </thead>
  <tbody class="t-small">
    <InvoiceDataRow />
    <InvoiceDataRow />
    <InvoiceDataRow />
    <InvoiceDataRow />
  </tbody>
</table>

<style>
  .invoices {
    margin-top: 60px;

    @media (--screen-smaller-max) {
      display: block;

      & thead {
        display: none;
      }

      & tbody {
        display: block;
      }
    }
  }
</style>
