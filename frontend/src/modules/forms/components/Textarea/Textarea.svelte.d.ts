import { SvelteComponentTyped } from 'svelte';
import type { Validator, FormControl } from 'svelte-use-form';

export interface TextareaProps
  extends svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['textarea']> {
  size: InputSize;
  validate?: Validator[];
  labelClass?: string;
  description?: string;
  field: FormControl;
}

export interface TextareaSlots {
  hints: Slot;
}

export default class Textarea extends SvelteComponentTyped<
  TextareaProps,
  undefined,
  TextareaSlots
> {}
