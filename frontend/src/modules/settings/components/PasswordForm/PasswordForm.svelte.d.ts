import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';
export interface PasswordFormProps {
  ui: UiContainer;
}

export default class PasswordForm extends SvelteComponentTyped<
  PasswordFormProps,
  undefined,
  undefined
> {}
