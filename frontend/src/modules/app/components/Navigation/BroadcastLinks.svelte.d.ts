import { SvelteComponentTyped } from 'svelte';
export interface BroadcastLinksProps {
  changeSubNav: (value: SubMenu) => void;
}

export default class BroadcastLinks extends SvelteComponentTyped<
  BroadcastLinksProps,
  undefined,
  undefined
> {}
