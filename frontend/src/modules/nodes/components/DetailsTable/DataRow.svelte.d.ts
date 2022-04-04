import { SvelteComponentTyped } from 'svelte';

export interface DataRowSlots {
  label: Slot;
  default: Slot;
}
export default class DataRow extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  DataRowSlots
> {}
