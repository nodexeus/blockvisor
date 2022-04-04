import { SvelteComponentTyped } from 'svelte';
import type { Validator, FormControl } from 'svelte-use-form';

export interface SelectProps
  extends svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['select']> {
  size: InputSize;
  validate?: Validator[];
  labelClass?: string;
  style: 'outline' | 'default';
  inputClass?: string;
  description?: string;
  field: FormControl;
  items: { value: string; label: string }[];
}

export interface SelectSlots {
  utilLeft: Slot;
  hints: Slot;
  label: Slot;
}

export default class Select extends SvelteComponentTyped<
  SelectProps,
  undefined,
  SelectSlots
> {}
