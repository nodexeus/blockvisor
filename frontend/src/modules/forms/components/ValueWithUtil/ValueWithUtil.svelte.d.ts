import { SvelteComponentTyped } from 'svelte';

export interface ValueWithUtilSlots {
  util: Slot;
  label: Slot;
  value: Slot;
}
export default class ValueWithUtil extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  ValueWithUtilSlots
> {}
