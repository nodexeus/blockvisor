import { SvelteComponentTyped } from 'svelte';

export interface StatsWithStateProps {
  state?: StatusState;
  value: string | number;
  id: string;
  changeOnZero?: boolean;
}

export interface StatsWithStateSlots {
  default: Slot;
}

export default class StatsWithState extends SvelteComponentTyped<
  StatsWithStateProps,
  Record<string, unknown>,
  StatsWithStateSlots
> {}
