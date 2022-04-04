import { SvelteComponentTyped } from 'svelte';

export interface SettingsNavEvents {
  click: (e: MouseEvent) => void;
}
export default class SettingsNav extends SvelteComponentTyped<
  Record<string, unknown>,
  SettingsNavEvents,
  undefined
> {}
