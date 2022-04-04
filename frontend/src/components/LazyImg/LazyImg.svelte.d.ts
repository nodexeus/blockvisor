import { SvelteComponentTyped } from 'svelte';

type LazyImgProps = svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['img']>;

export default class LazyImg extends SvelteComponentTyped<
  LazyImgProps,
  undefined,
  undefined
> {}
