import { SvelteComponentTyped } from 'svelte';
export interface HostGroupProps {
  hosts: string | number;
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
