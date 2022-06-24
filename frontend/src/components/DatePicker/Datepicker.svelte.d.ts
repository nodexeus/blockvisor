import { SvelteComponentTyped } from 'svelte';

export interface DatepickerProps {
  value: Date;
}

export default class Datepicker extends SvelteComponentTyped<
  MenuProps,
  undefined,
  undefined
> {}
