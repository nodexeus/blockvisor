import { SvelteComponentTyped } from 'svelte';

export interface HostDataRowSlots {
  title: Slot;
  default: Slot;
}
export default class HostDataRow extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  HostDataRowSlots
> {}
