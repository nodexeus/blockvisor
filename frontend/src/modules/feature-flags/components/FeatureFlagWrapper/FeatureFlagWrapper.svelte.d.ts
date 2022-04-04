import { SvelteComponentTyped } from 'svelte';
export interface FeatureFlagWrapperProps {
  shouldDisplay: boolean;
}

export interface FeatureFlagWrapperSlots {
  default: Slot;
  fallback: Slot;
}
export default class FeatureFlagWrapper extends SvelteComponentTyped<
  FeatureFlagWrapperProps,
  undefined,
  FeatureFlagWrapperSlots
> {}
