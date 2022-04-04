import { SvelteComponentTyped } from 'svelte';
export interface TabsProps {
  items: { value: string; label: string; component: svelte.JSX }[];
}

export default class Tabs extends SvelteComponentTyped<
  TabsProps,
  undefined,
  undefined
> {}
