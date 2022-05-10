import { SvelteComponentTyped } from 'svelte';
export interface HostGroupProps {
  selectedHosts: any;
  id?: string;
}

export interface HostGroupSlots {
  title: Slot;
  action: Slot;
}
export default class HostGroup extends SvelteComponentTyped<
  HostGroupProps,
  undefined,
  HostGroupSlots
> {}
