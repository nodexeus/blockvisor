<script lang="ts">
  import 'chartjs-adapter-date-fns';
  import { Chart, TimeScale, registerables, Interaction } from 'chart.js';
  import { CrosshairPlugin, Interpolate } from 'chartjs-plugin-crosshair';
  import { onMount } from 'svelte';

  const registerChart = () => {
    Interaction.modes.interpolate = Interpolate;
    Chart.register(CrosshairPlugin, TimeScale, ...registerables);
  };

  onMount(registerChart);
</script>
