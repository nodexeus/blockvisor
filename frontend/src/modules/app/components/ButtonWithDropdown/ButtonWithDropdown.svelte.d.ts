import type { ButtonProps } from 'components/Button/Button.svelte';
import { SvelteComponentTyped } from 'svelte';

export interface ButtonWithDropdownProps {
  buttonProps?: ButtonProps;
  iconButton?: boolean;
  position?: 'left' | 'right';
}
export interface ButtonWithDropdownEvents {
  click_outside: VoidFunction;
}
export interface ButtonWithDropdownSlots {
  label: Slot;
  content: Slot;
}
export default class ButtonWithDropdown extends SvelteComponentTyped<
  ButtonWithDropdownProps,
  ButtonWithDropdownEvents,
  ButtonWithDropdownSlots
> {}
