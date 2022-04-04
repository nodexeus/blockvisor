import { SvelteComponentTyped } from 'svelte';

export interface TitleWithContentBlockSlots {
  title: Slot;
  default: Slot;
}
export default class TitleWithContentBlock extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  TitleWithContentBlockSlots
> {}
