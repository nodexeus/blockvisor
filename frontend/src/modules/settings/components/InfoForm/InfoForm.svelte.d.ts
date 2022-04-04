import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';
export interface InfoFormProps {
  ui: UiContainer;
}

export default class InfoForm extends SvelteComponentTyped<
  InfoFormProps,
  undefined,
  undefined
> {}
