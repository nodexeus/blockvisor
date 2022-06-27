import { SvelteComponentTyped } from 'svelte';

export interface DatePickerProps {
  value: Date;
  field?: FormControl;
}

export default class Datepicker extends SvelteComponentTyped<
  DatePickerProps,
  undefined,
  undefined
> {}
