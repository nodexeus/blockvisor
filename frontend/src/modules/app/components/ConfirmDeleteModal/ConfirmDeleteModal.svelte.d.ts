import { SvelteComponentTyped } from 'svelte';
import type { Form } from 'svelte-use-form';
export interface ConfirmDeleteModalProps {
  targetValue?: string;
  isModalOpen?: boolean;
  handleModalClose: (e: MouseEvent) => void;
  id?: string;
  form: Form;
}
export interface ConfirmDeleteModalEvents {
  submit: (e: SubmitEvent) => void;
}
export interface ConfirmDeleteModalSlots {
  label: Slot;
  default: Slot;
}
export default class ConfirmDeleteModal extends SvelteComponentTyped<
  ConfirmDeleteModalProps,
  ConfirmDeleteModalEvents,
  ConfirmDeleteModalSlots
> {}
