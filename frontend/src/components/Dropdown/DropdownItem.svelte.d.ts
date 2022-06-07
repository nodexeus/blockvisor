import { SvelteComponentTyped } from 'svelte';
export interface DropdownItemProps
  extends svelte.JSX.HTMLAttributes<HTMLLinkElement | HTMLButtonElement> {
  as?: 'a' | 'button';
  size?: 'large' | 'small';
}
export interface DropdownItemSlots {
  default: Slot;
}

export interface DropdownItemEvents {
  click: VoidFunction;
}
export default class DropdownItem extends SvelteComponentTyped<
  DropdownItemProps,
  DropdownItemEvents,
  DropdownItemSlots
> {}
