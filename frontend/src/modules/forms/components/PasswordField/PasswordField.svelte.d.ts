import type { SvelteComponentTyped } from 'svelte';
import type { InputProps, InputSlots } from '../Input/Input.svelte.d.ts';

export interface PasswordFieldProps extends InputProps {
  type: 'password' | 'text';
}

export default class PasswordField extends SvelteComponentTyped<
  PasswordFieldProps,
  undefined,
  InputSlots
> {}
