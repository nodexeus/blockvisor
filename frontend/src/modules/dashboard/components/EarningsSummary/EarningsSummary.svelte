<script lang="ts">
  import TitleWithValue from 'components/TitleWithValue/TitleWithValue.svelte';
  import MinimalLineGraph from 'modules/charts/components/MinimalLineGraph/MinimalLineGraph.svelte';
  import { CONFIG_EARNINGS } from 'modules/charts/configs/chart-earnings';
  import ButtonSelector from 'modules/forms/components/ButtonSelector/ButtonSelector.svelte';

  var getDaysArray = function (s, e) {
    for (var a = [], d = new Date(s); d <= e; d.setDate(d.getDate() + 1)) {
      const randomDate = new Date(d);
      a.push({
        x: randomDate,
        y: Math.floor(Math.random() * (100 - 60 + 1)) + 60,
      });
    }
    return a;
  };

  const SAMPLE_OPTIONS = [
    {
      value: 'one-day',
      label: '1d',
    },
    {
      value: 'one-week',
      label: '1w',
      checked: true,
    },
    {
      value: 'one-month',
      label: '1m',
    },
    {
      value: 'three-months',
      label: '3m',
    },
    {
      value: 'six-months',
      label: '6m',
    },
    {
      value: 'one-year',
      label: '1y',
    },
    {
      value: 'all-time',
      label: 'All Time',
    },
  ];

  const PLACEHOLDER_DATA = getDaysArray(
    new Date('2022-06-01'),
    new Date('2022-07-01'),
  );
</script>

<ul class="u-list-reset earnings__stats">
  <li>
    <TitleWithValue id="js-total-earned" value={252.21}>
      <svelte:fragment slot="title">Total earned</svelte:fragment>
      <svelte:fragment slot="unit">HNT</svelte:fragment>
    </TitleWithValue>
  </li>
  <li>
    <TitleWithValue id="js-last-month" value={22.44}>
      <svelte:fragment slot="title">Last 30 days</svelte:fragment>
      <svelte:fragment slot="unit">HNT</svelte:fragment>
    </TitleWithValue>
  </li>
  <li>
    <TitleWithValue id="js-last-day" value={1.74}>
      <svelte:fragment slot="title">Last 24 hours</svelte:fragment>
      <svelte:fragment slot="unit">HNT</svelte:fragment>
    </TitleWithValue>
  </li>
</ul>

<div class="earnings__wrapper container--large">
  <MinimalLineGraph
    height="200"
    config={CONFIG_EARNINGS}
    data={[PLACEHOLDER_DATA]}
  >
    <svelte:fragment slot="label">Node earnings (USD)</svelte:fragment>
  </MinimalLineGraph>
</div>

<div class="earnings__controls">
  <ButtonSelector id="date-range" name="date-range" options={SAMPLE_OPTIONS}>
    <svelte:fragment>Date range</svelte:fragment>
  </ButtonSelector>
</div>

<style>
  .earnings {
    &__wrapper {
      padding-bottom: 20px;
      max-width: 1160px;
    }

    &__controls {
      margin-bottom: 40px;
    }

    &__stats {
      overflow-x: auto;
      display: flex;
      gap: 40px;
      padding-bottom: 40px;
    }
  }
</style>
