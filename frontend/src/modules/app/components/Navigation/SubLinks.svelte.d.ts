import { SvelteComponentTyped } from 'svelte';
export interface SubLinksProps {
  changeSubNav: (value: SubMenu) => void;
}

export interface SubLinksSlots {
  default: Slot;
}
export default class SubLinks extends SvelteComponentTyped<
  SubLinksProps,
  undefined,
  SubLinksSlots
> {}
