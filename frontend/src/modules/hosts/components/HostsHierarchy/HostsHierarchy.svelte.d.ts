import { SvelteComponentTyped } from 'svelte';
export interface HostsHiearchyProps {
  nodes: unknown[];
}
export interface HostsHiearchyEvents {
  click: (e: MouseEvent) => void;
}

export default class HostsHiearchy extends SvelteComponentTyped<
  HostsHiearchyProps,
  HostsHiearchyEvents,
  undefined
> {}
