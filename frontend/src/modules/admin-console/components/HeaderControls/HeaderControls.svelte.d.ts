import { SvelteComponentTyped } from 'svelte';

export interface HeaderControlsSlots {
  action: Slot;
}
export default class HeaderControls extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  HeaderControlsSlots
> {}
