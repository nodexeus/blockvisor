import { SvelteComponentTyped } from 'svelte';

export interface UserDetailsItemSlots {
  label: Slot;
  default: Slot;
}
export default class UserDetailsItem extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  UserDetailsItemSlots
> {}
