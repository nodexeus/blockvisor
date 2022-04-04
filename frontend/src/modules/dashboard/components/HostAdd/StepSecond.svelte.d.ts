import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';

export interface StepSecondProps {
  setStep: (step: number) => void;
  form: Form;
}
export default class StepSecond extends SvelteComponentTyped<
  StepSecondProps,
  undefined,
  undefined
> {}
