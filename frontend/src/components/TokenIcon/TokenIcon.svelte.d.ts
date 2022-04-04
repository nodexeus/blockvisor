import { SvelteComponentTyped } from 'svelte';
export interface TokenIconProps {
  icon: TokenCode;
}
export default class TokenIcon extends SvelteComponentTyped<
  TokenIconProps,
  undefined,
  undefined
> {}
