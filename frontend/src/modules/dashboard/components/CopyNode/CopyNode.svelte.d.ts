import { SvelteComponentTyped } from 'svelte';
export interface CopyNodeProps {
  value: string;
}

export interface CopyNodeSlots {
  default: Slot;
}

export default class CopyNode extends SvelteComponentTyped<
  CopyNodeProps,
  undefined,
  CopyNodeSlots
> {}
