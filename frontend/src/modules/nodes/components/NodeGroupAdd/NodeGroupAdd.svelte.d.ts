import { SvelteComponentTyped } from 'svelte';
export interface NodeGroupAddEvents {
  click: (MouseEvent) => void;
}
export interface NodeGroupAddSlots {
  default: Slot;
}
export default class NodeGroupAdd extends SvelteComponentTyped<
  svelte.JSX.HTMLAttributes<HTMLButtonElement>,
  NodeGroupAddEvents,
  NodeGroupAddSlots
> {}
