import { SvelteComponentTyped } from 'svelte';

export interface MenuProps {
  menuClass: string;
  handleClick: svelte.JSX.EventHandler<MouseEvent>;
}

export default class Menu extends SvelteComponentTyped<
  MenuProps,
  undefined,
  undefined
> {}
