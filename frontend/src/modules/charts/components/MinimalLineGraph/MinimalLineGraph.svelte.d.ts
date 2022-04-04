import type { ChartConfiguration, ChartData } from 'chart.js';
import { SvelteComponentTyped } from 'svelte';
export interface MinimalLineGraphProps
  extends svelte.JSX.HTMLAttributes<HTMLCanvasElement> {
  config: ChartConfiguration;
  data: ChartData<'line'>['datasets']['data'][];
}

export interface MinimalLineGraphSlots {
  label: Slot;
}
export default class MinimalLineGraph extends SvelteComponentTyped<
  MinimalLineGraphProps,
  undefined,
  MinimalLineGraphSlots
> {}
