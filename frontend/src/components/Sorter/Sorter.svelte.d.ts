import { SvelteComponentTyped } from 'svelte';
export interface SorterProps {
  callback?: (id: string, value: SorterValues) => void;
  id: string;
  active: { id: null | string; value: SorterValues };
}

export interface SorterSlots {
  default: Slot;
}

export default class Sorter extends SvelteComponentTyped<
  SorterProps,
  undefined,
  SorterSlots
> {}
