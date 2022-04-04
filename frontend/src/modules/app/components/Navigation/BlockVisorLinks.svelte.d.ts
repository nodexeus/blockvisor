import { SvelteComponentTyped } from 'svelte';
export interface MainLinksProps {
  changeSubNav: (value: SubMenu) => void;
}

export default class MainLinks extends SvelteComponentTyped<
  MainLinksProps,
  undefined,
  undefined
> {}
