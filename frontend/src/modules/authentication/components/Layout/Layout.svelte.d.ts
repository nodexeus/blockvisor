import { SvelteComponentTyped } from 'svelte';
export interface LayoutProps {
  title: string;
}
export interface LayoutSlots {
  default: Slot;
}
export default class Layout extends SvelteComponentTyped<
  LayoutProps,
  undefined,
  LayoutSlots
> {}
