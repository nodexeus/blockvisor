import { SvelteComponentTyped } from 'svelte';
export interface StateIconProps {
  status: HostState | NodeState;
}

export default class StateIcon extends SvelteComponentTyped<
  StateIconProps,
  undefined,
  undefined
> {}
