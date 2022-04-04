<script lang="ts">
  import Graph from '../Graph/Graph.svelte';
  import { createGradient, getPaddedMax } from '../../utils';
  import 'chartjs-adapter-date-fns';
  import type { Chart } from 'chart.js';

  export let config;
  export let data;

  const init = (chart: Chart<'line'>) => {
    chart.options.scales.y.suggestedMax = getPaddedMax(data);
    const gradientFill = createGradient(chart);
    chart.options.backgroundColor = gradientFill;
    chart.update();
  };
</script>

<Graph {init} {data} {config} {...$$restProps}>
  <slot name="label" slot="label" />
</Graph>
