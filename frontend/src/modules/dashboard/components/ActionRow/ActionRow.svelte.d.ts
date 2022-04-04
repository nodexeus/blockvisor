import { SvelteComponentTyped } from 'svelte';

export interface ActionRowSlots {
  title: Slot;
  description: Slot;
  action: Slot;
}
export default class ActionRow extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  ActionRowSlots
> {}
