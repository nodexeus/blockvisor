<script>
  import { fade } from 'svelte/transition';
  import { pageTransition } from 'consts/animations';
  import DatePicker from 'components/DatePicker/DatePicker.svelte';

  let date = new Date()
</script>

<section in:fade|local={pageTransition}>
  <DatePicker value={date} />
<!--   <DashboardCard type="full" important>
    <p slot="title">test</p>
    <p slot="value">540</p>
  </DashboardCard>
  <DashboardCard type="half" important>
    <p slot="title">test</p>
    <p slot="value">540</p>
  </DashboardCard>
  <DashboardCard type="quarter">
    <p slot="title">test</p>
    <p slot="value">540</p>
  </DashboardCard>
  <DashboardCard type="quarter">
    <p slot="title">test</p>
    <p slot="value">540</p>
  </DashboardCard> -->
</section>

<style>
  .dashboard {
    margin: 60px 0;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-gap: 12px;
  }

  .dashboard-card {
    border-radius: 8px;
    padding: 12px;
    background: rgba(248, 250, 246, 0.03);
  }

  .dashboard-card--full {
    grid-column: 1 / -1;
  }

  .dashboard-card--half {
    grid-column: span 2;
  }

  .dashboard-card--quarter {
    grid-column: span 1;
  }
</style>
