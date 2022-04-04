import type { ChartConfiguration } from 'chart.js';
import { SvelteComponentTyped } from 'svelte';
export interface GraphProps
  extends svelte.JSX.HTMLAttributes<HTMLCanvasElement> {
  config: ChartConfiguration;
  init: (chart: Chart) => void;
}

export interface GraphSlots {
  label: Slot;
}
export default class Graph extends SvelteComponentTyped<
  GraphProps,
  undefined,
  GraphSlots
> {}
