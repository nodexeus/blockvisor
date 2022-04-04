import { SvelteComponentTyped } from 'svelte';

export interface DataStateProps {
  state: HostState | NodeState;
}

export default class DataState extends SvelteComponentTyped<
  DataStateProps,
  undefined,
  undefined
> {}
