import { SvelteComponentTyped } from 'svelte';

export interface HeaderSearchProps {
  handleClose: VoidFunction;
  handleOpen: VoidFunction;
  isOpen: boolean;
}

export interface HeaderSearchEvents {
  click_outside: MouseEvent;
}

export interface HeaderSearchSlots {
  default: Slot;
}

export default class HeaderSearch extends SvelteComponentTyped<
  HeaderSearchProps,
  HeaderSearchEvents,
  HeaderSearchSlots
> {}
