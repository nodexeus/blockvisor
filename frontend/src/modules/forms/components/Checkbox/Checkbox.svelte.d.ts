import type { SvelteComponentTyped } from 'svelte';
import type { Validator, FormControl } from 'svelte-use-form';
export interface CheckboxProps
  extends svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['input']> {
  validate?: Validator[];
  description?: string;
  field: FormControl;
  formTouched?: boolean;
}

export interface CheckboxSlots {
  hints: Slot;
  default: Slot;
}

export default class Checkbox extends SvelteComponentTyped<
  CheckboxProps,
  undefined,
  CheckboxSlots
> {}
