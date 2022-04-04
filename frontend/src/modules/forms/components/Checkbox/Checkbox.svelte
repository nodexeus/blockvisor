<script lang="ts">
  import { afterUpdate } from 'svelte';
  import { validators } from 'svelte-use-form';

  import { isObjectEmpty } from 'utils';
  import InputHints from '../InputHints/InputHints.svelte';

  export let validate = [];
  export let field;
  export let description = '';
  export let formTouched = false;

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field?.touched || isObjectEmpty(field?.errors));

  afterUpdate(checkIfValid);

  $: checkboxClasses = [
    'checkbox',
    `checkbox--${isValid ? 'valid' : 'error'}`,
  ].join(' ');
</script>

<input
  use:validators={validate}
  class:checkbox__input--touched={formTouched}
  class="checkbox__input visually-hidden"
  id={$$props.name}
  type="checkbox"
  {...$$restProps}
/>
<label class={checkboxClasses} for={$$props.name}>
  <slot />
</label>

<InputHints
  name={$$props.name}
  disabled={$$props.disabled}
  {description}
  {isValid}
>
  <slot name="hints" />
</InputHints>

<style>
  .checkbox {
    display: inline-flex;
    align-items: flex-start;
    gap: 8px;
    color: theme(--color-text-3);
    cursor: pointer;
    user-select: none;
    @mixin font tiny;

    &::before {
      flex: 0 0 16px;
      content: '';
      display: block;
      width: 16px;
      height: 16px;
      border-radius: 4px;
      border: 1px solid theme(--color-border-2);
      background-color: theme(--color-input-background);
      background-repeat: no-repeat;
      background-position: 50% 50%;
      background-size: 8px;
    }
  }

  .checkbox__input:not(:disabled) + .checkbox:hover::before,
  .checkbox__input:not(:disabled) + .checkbox:focus::before,
  .checkbox__input:not(:disabled) + .checkbox:active::before {
    background-color: theme(--color-text-5-o10);
  }

  .checkbox__input:disabled + .checkbox {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .checkbox__input:focus:not(:hover) + .checkbox:before {
    outline: 1px dotted #212121;
    outline: 5px auto -webkit-focus-ring-color;
  }

  .checkbox__input--touched:invalid + .checkbox::before,
  .checkbox--error::before {
    border: 1px solid theme(--color-utility-warning);
  }

  .checkbox__input:checked + .checkbox::before {
    border: 1px solid theme(--color-utility-success);
    background-image: url("data:image/svg+xml,%3Csvg width='8' height='8' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='m1 5 2 2 4-6' stroke='%2385F550' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  }

  .checkbox__input:indeterminate + .checkbox::before {
    border-width: 0;
    background-image: url("data:image/svg+xml,%3Csvg fill='none' xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 2'%3E%3Cpath d='M1 1h8' stroke='%23B9B9B9' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  }
</style>
