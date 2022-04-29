import { SvelteComponentTyped } from 'svelte';

export interface PrivateRouteSlots {
  default: Slot;
}
export default class PrivateRoute extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  PrivateRouteSlots
> {}
