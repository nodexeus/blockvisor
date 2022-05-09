import { SvelteComponentTyped } from 'svelte';

export interface DataStateProps {
  status: HostState | NodeState;
}

export default class DataState extends SvelteComponentTyped<
  DataStateProps,
  undefined,
  undefined
> {}
