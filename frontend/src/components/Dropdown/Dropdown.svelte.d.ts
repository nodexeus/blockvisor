import { SvelteComponentTyped } from 'svelte';
export interface DropdownProps {
  isActive?: boolean;
}

export interface DropdownSlots {
  default: Slot;
}
export default class Dropdown extends SvelteComponentTyped<
  DropdownProps,
  undefined,
  DropdownSlots
> {}
