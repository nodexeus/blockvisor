import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';
export interface ModalProps {
  form?: Form;
  isActive: boolean;
  handleModalClose: (e: MouseEvent) => void;
  id?: string;
  size: 'normal' | 'large';
}

export interface ModalSlots {
  header: Slot;
  footer: Slot;
  defauklt: Slot;
}
export default class Modal extends SvelteComponentTyped<
  ModalProps,
  undefined,
  ModalSlots
> {}
