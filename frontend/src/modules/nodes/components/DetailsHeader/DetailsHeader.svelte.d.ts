import { SvelteComponentTyped } from 'svelte';
export interface DetailsHeaderProps {
  id: string;
  state: NodeState;
  form: Form;
}

export default class DetailsHeader extends SvelteComponentTyped<
  DetailsHeaderProps,
  undefined,
  undefined
> {}
