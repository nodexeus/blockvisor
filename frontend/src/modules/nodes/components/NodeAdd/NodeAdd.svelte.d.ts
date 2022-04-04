import { SvelteComponentTyped } from 'svelte';
export interface NodeAddProps {
  currentStep: number;
  setStep: (step: number) => void;
}

export default class NodeAdd extends SvelteComponentTyped<
  NodeAddProps,
  undefined,
  undefined
> {}
