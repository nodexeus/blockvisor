import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';

export interface AuthErrorsProps {
  ui: UiContainer;
}

export default class AuthErrors extends SvelteComponentTyped<
  AuthErrorsProps,
  undefined,
  undefined
> {}
