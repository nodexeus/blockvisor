import { SvelteComponentTyped } from 'svelte';
export interface ConfirmDeleteModalProps {
  isModalOpen?: boolean;
}
export interface ConfirmDeleteModalEvents {
  confirm: (e: SubmitEvent) => void;
  cancel: (e: SubmitEvent) => void;
}
export interface ConfirmDeleteModalSlots {
  content: Slot;
}
export default class ConfirmDeleteModal extends SvelteComponentTyped<
  ConfirmDeleteModalProps,
  ConfirmDeleteModalEvents,
  ConfirmDeleteModalSlots
> {}
