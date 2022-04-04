import { SvelteComponentTyped } from 'svelte';

export interface DropdownListSlots {
  default: Slot;
}
export default class DropdownList extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  DropdownListSlots
> {}
