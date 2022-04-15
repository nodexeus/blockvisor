import { SvelteComponentTyped } from 'svelte';

export interface PublicRouteSlots {
  default: Slot;
}
export default class PublicRoute extends SvelteComponentTyped<
  Record<string, unknown>,
  undefined,
  PublicRouteSlots
> {}
