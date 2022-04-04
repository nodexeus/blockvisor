import type { SvelteComponentTyped } from 'svelte';
import type { Validator, FormControl } from 'svelte-use-form';
export interface InputProps
  extends svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['input']> {
  size?: InputSize;
  validate?: Validator[];
  labelClass?: string;
  description?: string;
  field: FormControl;
}

export interface InputEvents {
  keyup: (e: KeyboardEvent) => void;
}

export interface InputSlots {
  iconLeft: Slot;
  iconRight: Slot;
  hints: Slot;
  label: Slot;
}

export default class Input extends SvelteComponentTyped<
  InputProps,
  InputEvents,
  InputSlots
> {}
