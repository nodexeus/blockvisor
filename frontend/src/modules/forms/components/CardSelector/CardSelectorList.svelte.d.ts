import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';
export interface CardSelectorListProps {
  form: Form;
  id?: string;
}

export interface CardSelectorListSlots {
  label: Slot;
  default: Slot;
}
export default class CardSelectorList extends SvelteComponentTyped<
  CardSelectorListProps,
  undefined,
  CardSelectorListSlots
> {}
