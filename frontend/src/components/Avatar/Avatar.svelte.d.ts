import { SvelteComponentTyped } from 'svelte';

export interface AvatarProps {
  src?: string;
  fullName?: string;
  size?: AvatarSizes;
}

export interface AvatarEvents {
  click: MouseEvent;
}

export default class Avatar extends SvelteComponentTyped<
  AvatarProps,
  AvatarEvents,
  undefined
> {}
