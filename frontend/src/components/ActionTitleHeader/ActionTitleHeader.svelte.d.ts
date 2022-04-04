import { SvelteComponentTyped } from 'svelte';
export interface ActionTitleHeaderProps {
  className?: string;
}

export interface ActionTitleHeaderSlots {
  title: Slot;
  action: Slot;
}
export default class ActionTitleHeader extends SvelteComponentTyped<
  ActionTitleHeaderProps,
  undefined,
  ActionTitleHeaderSlots
> {}
