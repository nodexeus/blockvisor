import { SvelteComponentTyped } from 'svelte';
export interface TitleWithValueProps {
  id: string;
  value: number | string;
}

export interface TitleWithValueSlots {
  title: Slot;
  unit: Slot;
}
export default class TitleWithValue extends SvelteComponentTyped<
  TitleWithValueProps,
  undefined,
  TitleWithValueSlots
> {}
