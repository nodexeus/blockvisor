import { SvelteComponentTyped } from 'svelte';
export interface DropdownItemProps
  extends svelte.JSX.HTMLAttributes<HTMLLinkElement | HTMLButtonElement> {
  as?: 'a' | 'button';
}
export interface DropdownItemSlots {
  default: Slot;
}
export default class DropdownItem extends SvelteComponentTyped<
  DropdownItemProps,
  undefined,
  DropdownItemSlots
> {}
