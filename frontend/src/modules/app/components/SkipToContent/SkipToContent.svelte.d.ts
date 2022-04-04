import { SvelteComponentTyped } from 'svelte';
export interface SkipToContentProps
  extends svelte.JSX.HTMLAttributes<HTMLAnchorElement> {
  target: string;
}

export interface SkipToContentSlots {
  default: Slot;
}
export default class SkipToContent extends SvelteComponentTyped<
  SkipToContentProps,
  undefined,
  SkipToContentSlots
> {}
