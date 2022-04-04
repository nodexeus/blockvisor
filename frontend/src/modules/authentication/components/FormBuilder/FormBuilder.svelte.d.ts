import type { UiContainer } from '@ory/kratos-client';
import { SvelteComponentTyped } from 'svelte';
import type { Form, Validator } from 'svelte-use-form';
export interface FormBuilderProps {
  node: UiContainer['nodes'];
  form: Form;
  validationMapper?: { [key: string]: Validator }[];
  hasPasswordConfirm?: boolean;
}

export interface FormBuilderSlots {
  'submit-button-label': Slot;
}

export default class FormBuilder extends SvelteComponentTyped<
  FormBuilderProps,
  undefined,
  FormBuilderSlots
> {}
