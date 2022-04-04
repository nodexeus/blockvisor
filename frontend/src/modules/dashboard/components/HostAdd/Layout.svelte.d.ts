import { SvelteComponentTyped } from 'svelte';
export interface LayoutProps {
  title: string;
  size: 'medium' | 'large';
}
export interface LayoutEvents {
  outroend: svelte.JSX.DOMAttributes<HTMLElement>['onoutroend'];
}
export interface LayoutSlots {
  default: Slot;
}

export default class Layout extends SvelteComponentTyped<
  LayoutProps,
  LayoutEvents,
  LayoutSlots
> {}
