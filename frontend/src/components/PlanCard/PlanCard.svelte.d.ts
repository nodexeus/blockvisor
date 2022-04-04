import { SvelteComponentTyped } from 'svelte';
export interface PlanCardProps {
  highlight?: boolean;
  plan?: string[];
}

export interface PlanCardSlots {
  title: Slot;
  fee: Slot;
  description: Slot;
  action: Slot;
  note: Slot;
  featured: Slot;
}

export default class PlanCard extends SvelteComponentTyped<
  PlanCardProps,
  undefined,
  PlanCardSlots
> {}
