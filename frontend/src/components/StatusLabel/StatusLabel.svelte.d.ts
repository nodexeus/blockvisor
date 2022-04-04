import { SvelteComponentTyped } from 'svelte';
export interface StatusLabelProps {
  status?: 'default';
}

export interface StatusLabelSlots {
  default: Slot;
}
export default class StatusLabel extends SvelteComponentTyped<
  StatusLabelProps,
  undefined,
  StatusLabelSlots
> {}
