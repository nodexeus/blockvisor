import { SvelteComponentTyped } from 'svelte';

export interface HostFormDataSlots {
  label: Slot;
  value: Slot;
}

export default class HostFormData extends SvelteComponentTyped<
  Record<string, unknown>,
  Record<string, unknown>,
  HostFormDataSlots
> {}
