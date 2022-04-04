import { SvelteComponentTyped } from 'svelte';

export interface StepListSlots {
  default: Slot;
}

export default class StepList extends SvelteComponentTyped<
  Record<string, unknown>,
  Record<string, unknown>,
  StepListSlots
> {}
