import { SvelteComponentTyped } from 'svelte';

export interface ContentColumnProps {
  headerSize?: number;
  contentSize?: number;
  id?: string;
}

export interface ContentColumnSlots {
  header: Slot;
  content: Slot;
}

export default class ContentColumn extends SvelteComponentTyped<
  ContentColumnProps,
  undefined,
  ContentColumnSlots
> {}
