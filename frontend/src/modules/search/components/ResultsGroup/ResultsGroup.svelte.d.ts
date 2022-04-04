import { SvelteComponentTyped } from 'svelte';
export interface ResultsGroupProps {
  items?: { title: string; description: string }[];
}

export interface ResultsGroupSlots {
  title: Slot;
  icon: Slot;
}
export default class ResultsGroup extends SvelteComponentTyped<
  ResultsGroupProps,
  undefined,
  ResultsGroupSlots
> {}
