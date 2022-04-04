import { SvelteComponentTyped } from 'svelte';
export interface LayoutSlots {
  default: Slot;
}
export default class Layout extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  LayoutSlots
> {}
