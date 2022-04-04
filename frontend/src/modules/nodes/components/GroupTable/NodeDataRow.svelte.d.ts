import { SvelteComponentTyped } from 'svelte';
export interface NodeDataRowProps {
  token: string;
  id: string;
  ip: string;
  name: string;
  added: string;
  state: NodeState;
}

export default class NodeDataRow extends SvelteComponentTyped<
  NodeDataRowProps,
  undefined,
  undefined
> {}
