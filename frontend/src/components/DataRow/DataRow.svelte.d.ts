import { SvelteComponentTyped } from 'svelte';

export interface DataRowProps {
  state: HostState | NodeState;
}

export interface DataRowSlots {
  primary: Slot;
  secondary: Slot;
  action: Slot;
}

export default class DataRow extends SvelteComponentTyped<
  DataRowProps,
  undefined,
  DataRowSlots
> {}
