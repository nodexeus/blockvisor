import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';
export interface FormStateProps {
  values: Form['values'][];
  setStep: (step: number) => void;
  label: string;
  step: number | string;
}

export default class FormState extends SvelteComponentTyped<
  FormStateProps,
  undefined,
  undefined
> {}
