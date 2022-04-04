import { SvelteComponentTyped } from 'svelte';

export interface PillSlots {
  default: Slot;
}

export default class Pill extends SvelteComponentTyped<
  { removable?: boolean; transition: SvelteTransitionConfig },
  Record<string, unknown>,
  PillSlots
> {}
