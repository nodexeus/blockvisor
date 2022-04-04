import { SvelteComponentTyped } from 'svelte';

export interface InputLabelProps {
  size?: InputSize;
  labelClass?: string;
  disabled: boolean;
  name: string;
}

export interface InputLabelSlots {
  default: Slot;
}

export default class InputLabel extends SvelteComponentTyped<
  InputLabelProps,
  undefined,
  InputLabelSlots
> {}
