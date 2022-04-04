import { SvelteComponentTyped } from 'svelte';
export interface InformationItemProps {
  items: { title: string; info: string }[];
}

export interface InformationItemSlots {
  default?: Slot;
}

export default class InformationItem extends SvelteComponentTyped<
  InformationItemProps,
  undefined,
  InformationItemSlots
> {}
