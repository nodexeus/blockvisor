import { SvelteComponentTyped } from 'svelte';
export interface LayoutTwoColumnProps {
  transition?: SvelteTransitionConfig;
}

export interface LayoutTwoColumnSlots {
  sidebar: Slot;
  default: Slot;
}
export default class LayoutTwoColumn extends SvelteComponentTyped<
  LayoutTwoColumnProps,
  undefined,
  LayoutTwoColumnSlots
> {}
