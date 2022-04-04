import { SvelteComponentTyped } from 'svelte';

export interface BackButtonSlots {
  default: Slot;
}
export default class BackButton extends SvelteComponentTyped<
  ButtonProps,
  undefined,
  BackButtonSlots
> {}
