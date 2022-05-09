import { SvelteComponentTyped } from 'svelte';

export interface NodeHierarchyEvents {
  click: (e: MouseEvent) => void;
}

export default class NodeHierarchy extends SvelteComponentTyped<
  NodeHierarchyEvents,
  undefined
> {}
