import { SvelteComponentTyped } from 'svelte';
export interface SubscriptionWrapperProps {
  hasSubscription?: boolean;
}

export interface SubscriptionWrapperSlots {
  default: Slot;
}
export default class SubscriptionWrapper extends SvelteComponentTyped<
  SubscriptionWrapperProps,
  undefined,
  SubscriptionWrapperSlots
> {}
