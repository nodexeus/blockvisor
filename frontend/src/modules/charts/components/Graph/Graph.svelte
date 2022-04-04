<script lang="ts">
  import { onMount } from 'svelte';
  import { Chart } from 'chart.js';
  import { createDatasets } from 'modules/charts/utils/create-datasets';

  export let init;
  export let config;
  export let data;

  let canvas;
  let chart;

  const initChart = () => {
    chart = new Chart(canvas?.getContext('2d'), {
      ...config,
      data: {
        datasets: createDatasets(data, 'line'),
      },
    });
    init(chart);
  };

  onMount(initChart);
</script>

<figure class="chart">
  {#if $$slots.label}
    <figcaption class="chart__title t-uppercase t-color-text-3 t-microlabel">
      <slot name="label" />
    </figcaption>
  {/if}
  <div>
    <canvas class="chart__canvas" {...$$restProps} bind:this={canvas} />
  </div>
</figure>

<style>
  .chart {
    position: relative;

    &__title {
      margin-bottom: 20px;
    }

    &__canvas {
      cursor: crosshair;
      max-width: 100%;
    }

    & :global(.chart__tooltip) {
      pointer-events: none;
      position: absolute;
      white-space: nowrap;
      padding: 0 12px;
      text-transform: uppercase;

      & :global(.label) {
        display: block;
      }

      & :global(.value) {
        display: block;
        padding-bottom: 8px;
      }
    }
  }
</style>
