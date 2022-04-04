import { SvelteComponentTyped } from 'svelte';

export interface NavItemProps
  extends svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['a']> {
  urlAlias?: string;
  checkIfParent?: boolean;
}

export interface NavItemSlots {
  icon: Slot;
  label: Slot;
  default: Slot;
}

export interface NavItemEvents {
  click: MouseEvent;
}

export default class NavItem extends SvelteComponentTyped<
  NavItemProps,
  NavItemEvents,
  NavItemSlots
> {}
