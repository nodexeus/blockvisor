import { SvelteComponentTyped } from 'svelte';
export interface IconButtonProps {
  size?: IconButtonSize;
  style?: ButtonStyle;
  border?: ButtonBorder;
  cssCustom?: string;
  asLink?: boolean;
}

export interface IconButtonEvents {
  click: (e: MouseEvent) => void;
}

export interface IconButtonSlots {
  default: Slot;
}

export default class IconButton extends SvelteComponentTyped<
  IconButtonProps,
  IconButtonEvents,
  IconButtonSlots
> {}
