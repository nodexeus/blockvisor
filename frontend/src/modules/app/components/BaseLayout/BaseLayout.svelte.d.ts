import { SvelteComponentTyped } from 'svelte';

export interface BaseLayoutSlots {
  default: Slot;
}
export default class BaseLayout extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  BaseLayoutSlots
> {}
