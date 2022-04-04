import { SvelteComponentTyped } from 'svelte';
export interface HosetDataRowProps {
  name: string;
  status: HostState;
  url: string;
  ip: string;
  location: string;
}

export default class HosetDataRow extends SvelteComponentTyped<
  HosetDataRowProps,
  undefined,
  undefined
> {}
