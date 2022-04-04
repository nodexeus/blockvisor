import { SvelteComponentTyped } from 'svelte';
export interface SubscriptionToggleProps extends ButtonProps {
  isActive?: boolean;
}
export interface SubscriptionToggleEvents {
  click: (e: MouseEvent) => void;
}
export interface SubscriptionToggleSlots {
  default: Slot;
}
export default class SubscriptionToggle extends SvelteComponentTyped<
  SubscriptionToggleProps,
  SubscriptionToggleEvents,
  SubscriptionToggleSlots
> {}
