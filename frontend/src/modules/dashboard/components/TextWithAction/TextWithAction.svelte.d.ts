import { SvelteComponentTyped } from 'svelte';

export interface TextWithActionSlots {
  action: Slot;
  default: Slot;
}

export default class TextWithAction extends SvelteComponentTyped<
  Record<string, unknown>,
  Record<string, unknown>,
  TextWithActionSlots
> {}
