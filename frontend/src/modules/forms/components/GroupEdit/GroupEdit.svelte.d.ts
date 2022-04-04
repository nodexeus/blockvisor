import { SvelteComponentTyped } from 'svelte';
export interface GroupEditProps {
  value?: string;
  handleConfirm: VoidFunction;
}

export default class GroupEdit extends SvelteComponentTyped<
  GroupEditProps,
  undefined,
  undefined
> {}
