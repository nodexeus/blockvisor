import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';

export interface AuthNotesProps {
  ui: UiContainer;
}

export default class AuthNotes extends SvelteComponentTyped<
  AuthNotesProps,
  undefined,
  undefined
> {}
