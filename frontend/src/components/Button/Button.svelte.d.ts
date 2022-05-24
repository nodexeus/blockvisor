import { SvelteComponentTyped } from 'svelte';

export interface ButtonProps
  extends Partial<svelte.JSX.HTMLAttributes<HTMLElementTagNameMap['button']>> {
  size?: ButtonSize;
  style?: ButtonStyle;
  border?: ButtonBorder;
  display?: ButtonDisplay;
  cssCustom?: string;
  asLink?: boolean;
  handleClick: () => void;
}

export interface ButtonEvents {
  click: MouseEvent;
}

export interface ButtonSlots {
  default: Slot;
}

export default class Button extends SvelteComponentTyped<
  ButtonProps,
  ButtonEvents,
  ButtonSlots
> {}
