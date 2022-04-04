import type { AvatarProps } from 'components/Avatar/Avatar.svelte';
import { SvelteComponentTyped } from 'svelte';

export interface SideNavProps extends AvatarProps {
  handleClickOutside: () => void;
}

export interface SideNavEvents {
  click_outside: MouseEvent;
}

export interface SideNavSlots {
  default: Slot;
}

export default class SideNav extends SvelteComponentTyped<
  SideNavProps,
  SideNavEvents,
  SideNavSlots
> {}
