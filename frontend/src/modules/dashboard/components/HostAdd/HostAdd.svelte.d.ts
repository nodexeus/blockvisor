import { SvelteComponentTyped } from 'svelte';

export interface HostAddProps {
  currentStep: number;
  setStep: (n: number) => void;
}

export interface HostAddPropsSlots {
  default: Slot;
}

export default class HostAdd extends SvelteComponentTyped<
  HostAddProps,
  undefined,
  HostFormDataSlots
> {}
