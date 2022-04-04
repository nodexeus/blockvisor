import { SvelteComponentTyped } from 'svelte';
import type { Validator } from 'svelte-use-form';
export interface ToggleProps {
  validate?: Validator[];
  field: FormControl;
  formTouched?: boolean;
}

export interface ToggleSlots {
  default: Slot;
  hints: Slot;
  description: Slot;
}
export default class Toggle extends SvelteComponentTyped<
  ToggleProps,
  undefined,
  ToggleSlots
> {}
