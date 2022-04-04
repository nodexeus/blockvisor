import { SvelteComponentTyped } from 'svelte';
export interface AppButtonProps
  extends svelte.JSX.HTMLAttributes<HTMLButtonElement> {
  activeApp: number;
  value: number | string;
}

export interface AppButtonSlots {
  default: Slot;
}
export default class AppButton extends SvelteComponentTyped<
  AppButtonProps,
  AppButtonEvents,
  AppButtonSlots
> {}
