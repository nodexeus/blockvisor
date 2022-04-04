import { SvelteComponentTyped } from 'svelte';
export interface FormSectionProps {
  title: string;
}

export interface FormSectionSlots {
  default: Slot;
}
export default class FormSection extends SvelteComponentTyped<
  FormSectionProps,
  undefined,
  FormSectionSlots
> {}
