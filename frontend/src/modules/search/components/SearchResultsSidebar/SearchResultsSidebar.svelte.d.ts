import { SvelteComponentTyped } from 'svelte';
export interface SearchResultsSidebarProps {
  isActive?: boolean;
  handleClose?: VoidFunction;
}

export default class SearchResultsSidebar extends SvelteComponentTyped<
  SearchResultsSidebarProps,
  undefined,
  undefined
> {}
