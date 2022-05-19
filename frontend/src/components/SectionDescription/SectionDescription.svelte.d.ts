import { SvelteComponentTyped } from 'svelte';

export interface SectionDescriptionProps {
  value: Slot;
}

export interface SectionDescriptionSlots {
  default: Slot;
}

export default class SectionDescription extends SvelteComponentTyped<
  Record<string, unknown>,
  SectionDescriptionProps,
  SectionDescriptionSlots
> {}
