import { SvelteComponentTyped } from 'svelte';

export interface InputHintProps {
  isValid: boolean;
  description: string;
  disabled: boolean;
  name: string;
}

export interface InputHintSlots {
  default: Slot;
}

export default class InputHint extends SvelteComponentTyped<
  InputHintProps,
  undefined,
  InputHintSlots
> {}
