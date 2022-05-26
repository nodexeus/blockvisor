import { SvelteComponentTyped } from 'svelte';
import type { Organisation } from '../models/Organisation';

interface Props {
  item: Organisation;
  index: number;
}

export default class OrganisationDataRow extends SvelteComponentTyped<
  Props,
  undefined,
  undefined
> {}
