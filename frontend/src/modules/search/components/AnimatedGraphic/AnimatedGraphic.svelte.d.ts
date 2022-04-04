import { SvelteComponentTyped } from 'svelte';
export interface AnimatedGraphicProps {
  rotationAnimation?: boolean;
  id?: string;
}

export default class AnimatedGraphic extends SvelteComponentTyped<
  AnimatedGraphicProps,
  undefined,
  undefined
> {}
