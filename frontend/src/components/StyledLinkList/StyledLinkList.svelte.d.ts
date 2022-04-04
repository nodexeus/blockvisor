import { SvelteComponentTyped } from 'svelte';

export interface StyledLinkListProps {
  links: { text: string; href: string }[];
  title: string;
}

export default class StyledLinkList extends SvelteComponentTyped<
  StyledLinkListProps,
  undefined,
  undefined
> {}
