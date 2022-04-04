import { SvelteComponentTyped } from 'svelte';

export interface LoadingSpinnerProps {
  size?: string;
  id: string;
}

export default class LoadingSpinner extends SvelteComponentTyped<
  LoadingSpinnerProps,
  undefined,
  undefined
> {}
