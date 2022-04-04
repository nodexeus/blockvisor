import { SvelteComponentTyped } from 'svelte';

export interface PaginationProps {
  pages: number;
  perPage: number;
  itemsTotal: number;
  currentPage: number;
  onPageChange: (page: number) => void;
}

export default class Pagination extends SvelteComponentTyped<
  PaginationProps,
  undefined,
  undefined
> {}
