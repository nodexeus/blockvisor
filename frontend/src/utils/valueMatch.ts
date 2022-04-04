import type { Form } from 'svelte-use-form';

export const valueMatch = (value: string, form: Form) => {
  if (value && value === form?.targetValue?.initial) {
    return null;
  }

  return { valueMatch: 'Enter correct value' };
};
