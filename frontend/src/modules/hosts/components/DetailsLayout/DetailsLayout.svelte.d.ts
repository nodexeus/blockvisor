import { SvelteComponentTyped } from 'svelte';

export interface DetailsLayoutSlots {
  nav: Slot;
  default: Slot;
}
export default class DetailsLayout extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  DetailsLayoutSlots
> {}
