import { SvelteComponentTyped } from 'svelte';

export interface QuoteBlockProps {
  author?: string;
  role?: string;
}

export interface QuoteBlockSlots {
  content: Slot;
  image: Slot;
}

export default class QuoteBlock extends SvelteComponentTyped<
  QuoteBlockProps,
  undefined,
  QuoteBlockSlots
> {}
