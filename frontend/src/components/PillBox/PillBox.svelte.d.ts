import { SvelteComponentTyped } from 'svelte';

export interface PillBoxSlots {
  default: Slot;
}

export default class PillBox extends SvelteComponentTyped<
  Record<string, unknown>,
  Record<string, unknown>,
  PillBoxSlots
> {}
