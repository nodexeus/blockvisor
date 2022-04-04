import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';
export interface NewValidatorFormProps {
  form: Form;
  setStep: (step: number) => void;
}

export default class NewValidatorForm extends SvelteComponentTyped<
  NewValidatorFormProps,
  undefined,
  undefined
> {}
