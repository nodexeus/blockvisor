import { SvelteComponentTyped } from 'svelte';
export interface BillingProps {
  hasActivePlan?: boolean;
}

export default class Billing extends SvelteComponentTyped<
  BillingProps,
  undefined,
  undefined
> {}
