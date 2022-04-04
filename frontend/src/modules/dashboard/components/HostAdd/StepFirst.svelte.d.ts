import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';

export interface StepFirstProps {
  setStep: (step: number) => void;
  form: Form;
}
export default class StepFirst extends SvelteComponentTyped<
  StepFirstProps,
  undefined,
  undefined
> {}
