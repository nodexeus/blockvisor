import { SvelteComponentTyped } from 'svelte';

export interface GroupTitleSlots {
  title: Slot;
  action: Slot;
  state: Slot;
}
export default class GroupTitle extends SvelteComponentTyped<
  Record<string, unknown>,
  Record<string, unknown>,
  GroupTitleSlots
> {}
