import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';

export interface StepThirdProps {
  setStep: (step: number) => void;
  form: Form;
}
export default class StepThird extends SvelteComponentTyped<
  StepThirdProps,
  undefined,
  undefined
> {}
