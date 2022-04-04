import { SvelteComponentTyped } from 'svelte';
export interface MapProps {
  style?: string;
  callback?: (L: L.Class, map: L.Map) => void;
}

export interface MapSlots {
  default: Slot;
}
export default class Map extends SvelteComponentTyped<
  MapProps,
  undefined,
  MapSlots
> {}
