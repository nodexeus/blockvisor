import type { Form } from 'svelte-use-form';

export const passwordMatchPassword = (value: string, form: Form) => {
  if (!value || !form?.['password'].value) {
    return null;
  }
  return value === form?.['password'].value
    ? null
    : { passwordMatch: 'Passwords do not match' };
};

export const passwordMatchConfirm = (value: string, form: Form) => {
  if (!value || !form.confirmPassword.value) {
    return null;
  }
  value === form.confirmPassword.value
    ? null
    : { passwordMatch: 'Passwords do not match' };
};
