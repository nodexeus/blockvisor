import { SvelteComponentTyped } from 'svelte';
export interface DetailsTableProps {
  data: NodeDetails;
}

export default class DetailsTable extends SvelteComponentTyped<
  DetailsTableProps,
  undefined,
  undefined
> {}
