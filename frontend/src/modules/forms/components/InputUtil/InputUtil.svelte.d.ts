import { SvelteComponentTyped } from 'svelte';

export interface InputUtilProps {
  position?: UtilPosition;
}

export interface InputUtilSlots {
  default: Slot;
}

export default class InputUtil extends SvelteComponentTyped<
  InputUtilProps,
  undefined,
  InputUtilSlots
> {}
