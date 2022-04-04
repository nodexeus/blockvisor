import type { AnimeParams, AnimeTimelineAnimParams } from 'animejs';
import { SvelteComponentTyped } from 'svelte';

export interface AnimateOnEntryProps {
  timeline?: AnimeTimelineAnimParams;
  beforeAnimation?: VoidFunction;
  once?: boolean;
  type?: keyof HTMLElementTagNameMap;
  rootMargin?: string;
  threshold?: number;
  animeOptions?: AnimeParams;
}

export interface AnimateOnEntrySlots {
  default: Slot;
}

export default class AnimateOnEntry extends SvelteComponentTyped<
  AnimateOnEntryProps,
  undefined,
  AnimateOnEntrySlots
> {}
