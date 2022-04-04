import { SvelteComponentTyped } from 'svelte';

export interface StatusListItemProps {
  content?: string;
  status?: string;
}

export interface StatusListItemSlots {
  default: Slot;
}

export default class StatusListItem extends SvelteComponentTyped<
  StatusListItemProps,
  undefined,
  StatusListItemSlots
> {}
