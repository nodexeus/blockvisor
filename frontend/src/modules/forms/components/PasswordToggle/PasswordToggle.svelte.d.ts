import type { SvelteComponentTyped } from 'svelte';

export interface ButtonEvents {
  click: MouseEvent;
}

export default class PasswordToggle extends SvelteComponentTyped<
  { activeType?: 'password' | 'text' },
  ButtonEvents,
  undefined
> {}
