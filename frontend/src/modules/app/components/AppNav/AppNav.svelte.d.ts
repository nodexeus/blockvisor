import { SvelteComponentTyped } from 'svelte';
export interface AppNavProps {
  callback: (newValue: number) => void;
}

export default class AppNav extends SvelteComponentTyped<
  AppNavProps,
  undefined,
  undefined
> {}
