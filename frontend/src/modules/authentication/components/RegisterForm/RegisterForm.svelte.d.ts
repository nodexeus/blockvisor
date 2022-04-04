import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';
export interface RegisterFormProps {
  ui: UiContainer;
  order?: string[];
}

export default class RegisterForm extends SvelteComponentTyped<
  RegisterFormProps,
  undefined,
  undefined
> {}
