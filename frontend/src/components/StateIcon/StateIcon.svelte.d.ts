import { SvelteComponentTyped } from 'svelte';
export interface StateIconProps {
  state: HostState | NodeState;
}

export default class StateIcon extends SvelteComponentTyped<
  StateIconProps,
  undefined,
  undefined
> {}
