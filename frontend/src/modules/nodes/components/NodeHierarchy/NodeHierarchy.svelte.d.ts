import { SvelteComponentTyped } from 'svelte';
export interface NodeHierarchyProps {
  nodes: unknown[];
}
export interface NodeHierarchyEvents {
  click: (e: MouseEvent) => void;
}

export default class NodeHierarchy extends SvelteComponentTyped<
  NodeHierarchyProps,
  NodeHierarchyEvents,
  undefined
> {}
