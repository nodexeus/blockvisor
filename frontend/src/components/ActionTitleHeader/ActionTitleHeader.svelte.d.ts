import { SvelteComponentTyped } from 'svelte';
export interface ActionTitleHeaderProps {
  className?: string;
}

export interface ActionTitleHeaderSlots {
  title: Slot;
  action: Slot;
  util: Slot;
}
export default class ActionTitleHeader extends SvelteComponentTyped<
  ActionTitleHeaderProps,
  undefined,
  ActionTitleHeaderSlots
> {}
