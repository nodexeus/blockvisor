import { SvelteComponentTyped } from 'svelte';
export interface NodeGroupProps {
  id?: string;
  nodes: any[];
}

export interface NodeGroupSlots {
  title: Slot;
  actions: Slot;
  label: Slot;
}
export default class NodeGroup extends SvelteComponentTyped<
  NodeGroupProps,
  undefined,
  NodeGroupSlots
> {}
