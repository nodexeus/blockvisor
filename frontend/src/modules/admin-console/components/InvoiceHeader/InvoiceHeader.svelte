<script>
  import Button from 'components/Button/Button.svelte';
  import StatusLabel from 'components/StatusLabel/StatusLabel.svelte';

  import IconCalendar from 'icons/calendar-12.svg';
  import IconDownload from 'icons/download-12.svg';
</script>

<header class="invoice-header container--medium-large">
  <div class="invoice-header__info">
    <h2 class="invoice-header__title">Invoice 00001/2022</h2>
    <StatusLabel>Unpaid</StatusLabel>
    <date class="t-microlabel invoice-header__date">
      <IconCalendar />
      02/24/2022
    </date>
  </div>

  <div class="invoice-header__actions">
    <Button style="primary" size="small">Pay Invoice</Button>
    <Button style="outline" size="small">
      <IconDownload />
      Download .PDF</Button
    >
  </div>
</header>

<style>
  .invoice-header {
    margin-top: 60px;
    margin-bottom: 60px;
    display: flex;
    justify-content: space-between;
    gap: 32px 80px;
    flex-wrap: wrap;

    &__actions {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    &__date {
      display: flex;
      gap: 8px;
      flex-basis: 100%;
      color: theme(--color-text-3);
    }

    &__info {
      flex-wrap: wrap;
      display: flex;
      gap: 12px;
      align-items: middle;

      @media (--screen-medium-small) {
        gap: 4px 12px;
      }
    }
  }
</style>
