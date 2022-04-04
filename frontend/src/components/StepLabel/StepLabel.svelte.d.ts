import { SvelteComponentTyped } from 'svelte';

export interface StepLabelProps {
  currentStep: number;
  step: number;
  setStep: (step: number) => void;
  callback?: (step: number) => void;
}

export interface StepLabelSlots {
  default: Slot;
}

export default class StepLabel extends SvelteComponentTyped<
  StepLabelProps,
  Record<string, unknown>,
  StepLabelSlots
> {}
