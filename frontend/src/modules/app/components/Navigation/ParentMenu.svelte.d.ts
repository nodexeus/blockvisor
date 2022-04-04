import { SvelteComponentTyped } from 'svelte';
export interface ParentMenuProps {
  isActive?: boolean;
}

export interface ParentMenuSlots {
  default: Slot;
}

export default class ParentMenu extends SvelteComponentTyped<
  ParentMenuProps,
  undefined,
  ParentMenuSlots
> {}
