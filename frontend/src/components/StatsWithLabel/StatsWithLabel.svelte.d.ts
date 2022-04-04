import { SvelteComponentTyped } from 'svelte';

export interface StatsWithLabelSlots {
  label: Slot;
  value: Slot;
}
export default class StatsWithLabel extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  StatsWithLabelSlots
> {}
