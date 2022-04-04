import { SvelteComponentTyped } from 'svelte';

export interface StatusListSlots {
  default: Slot;
}

export default class StatusList extends SvelteComponentTyped<
  undefined,
  undefined,
  StatusListSlots
> {}
