import { SvelteComponentTyped } from 'svelte';

export interface MasterLayoutSlots {
  default: Slot;
}

export default class MasterLayoutSlots extends SvelteComponentTyped<
  { fullName?: string; src?: string },
  Record<string, unknown>,
  MasterLayoutSlots
> {}
