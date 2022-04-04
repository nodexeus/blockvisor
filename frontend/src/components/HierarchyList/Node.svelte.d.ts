import { SvelteComponentTyped } from 'svelte';

export interface NodeSlots {
  default: Slot;
}

export interface NodeEvents {
  click: (e: MouseEvent) => void;
}

export default class Node extends SvelteComponentTyped<
  NavNodeProps,
  NodeEvents,
  NodeSlots
> {}
