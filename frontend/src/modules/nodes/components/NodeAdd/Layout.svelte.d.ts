import { SvelteComponentTyped } from 'svelte';
export interface LayoutProps {
  string: 'medium' | 'large';
  title: string;
}
export interface LayoutEvents {
  outroend: VoidFunction;
}
export interface LayoutSlots {
  default: Slot;
}
export default class Layout extends SvelteComponentTyped<
  LayoutProps,
  LayoutEvents,
  LayoutSlots
> {}
