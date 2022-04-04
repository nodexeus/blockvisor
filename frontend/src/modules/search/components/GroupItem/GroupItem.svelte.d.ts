import { SvelteComponentTyped } from 'svelte';
export interface GroupItemProps {
  title: string;
  description: string;
}

export interface GroupItemSlots {
  icon: Slot;
}
export default class GroupItem extends SvelteComponentTyped<
  GroupItemProps,
  undefined,
  GroupItemSlots
> {}
