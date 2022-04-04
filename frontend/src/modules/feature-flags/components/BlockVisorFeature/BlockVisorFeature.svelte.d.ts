import { SvelteComponentTyped } from 'svelte';
export interface BlockVisorFeatureProps {
  type?: 'overlay' | 'small';
}

export interface BlockVisorFeatureSlots {
  default: Slot;
}
export default class BlockVisorFeature extends SvelteComponentTyped<
  BlockVisorFeatureProps,
  undefined,
  BlockVisorFeatureSlots
> {}
