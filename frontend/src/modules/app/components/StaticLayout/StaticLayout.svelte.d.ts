import { SvelteComponentTyped } from 'svelte';

export interface StaticLayoutSlots {
  title: Slot;
  util: Slot;
  default: Slot;
}
export default class StaticLayout extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  StaticLayoutSlots
> {}
