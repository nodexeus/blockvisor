import { SvelteComponentTyped } from 'svelte';
export interface CardSelectorProps {
  disabled?: boolean;
}

export interface CardSelectorSlots {
  top: Slot;
  label: Slot;
  action: Slot;
}
export default class CardSelector extends SvelteComponentTyped<
  CardSelectorProps,
  undefined,
  CardSelectorSlots
> {}
