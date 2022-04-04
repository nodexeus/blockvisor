import { SvelteComponentTyped } from 'svelte';
export interface LogoutLinkSlots {
  default: Slot;
}
export default class LogoutLink extends SvelteComponentTyped<
  svelte.JSX.HTMLAttributes<HTMLLinkElement>,
  undefined,
  LogoutLinkSlots
> {}
