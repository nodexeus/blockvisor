import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';

export interface ForgotPasswordFormProps {
  ui: UiContainer;
  form: Form;
}

export default class ForgotPasswordForm extends SvelteComponentTyped<
  ForgotPasswordFormProps,
  undefined,
  undefined
> {}
