import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';
export interface LoginFormProps {
  ui: UiContainer;
}

export default class LoginForm extends SvelteComponentTyped<
  LoginFormProps,
  undefined,
  undefined
> {}
