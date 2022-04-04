<script lang="ts">
  import { afterUpdate } from 'svelte';
  import { validators } from 'svelte-use-form';

  import { isObjectEmpty } from 'utils';
  import InputHints from '../InputHints/InputHints.svelte';

  export let validate = [];
  export let field;
  export let formTouched = false;

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field.touched || isObjectEmpty(field.errors));

  afterUpdate(checkIfValid);

  $: toggleClasses = ['toggle', `toggle--${isValid ? 'valid' : 'error'}`].join(
    ' ',
  );
</script>

<input
  use:validators={validate}
  class:toggle__input--touched={formTouched}
  class="toggle__input visually-hidden"
  id={$$props.name}
  type="checkbox"
  {...$$restProps}
/>

<label class={toggleClasses} for={$$props.name}>
  <div
    class="toggle__label"
    on:click={(e) => {
      e.preventDefault();
      e.stopPropagation();
    }}
  >
    <slot />
    {#if $$slots.description}
      <small class="t-tiny toggle__description">
        <slot name="description" />
      </small>
    {/if}
  </div>
</label>

<InputHints
  description=""
  name={$$props.name}
  disabled={$$props.disabled}
  {isValid}
>
  <slot name="hints" />
</InputHints>

<style>
  .toggle {
    cursor: pointer;
    user-select: none;
    display: flex;
    gap: 20px;
    position: relative;

    &__label {
      cursor: auto;
    }

    &__description {
      display: block;
      color: theme(--color-text-3);
    }

    &::before {
      margin-top: 2px;
      content: '';
      flex: 0 0 32px;
      border: 1px solid theme(--color-border-2);
      display: block;
      width: 32px;
      height: 20px;
      border-radius: 24px;
      transition: border-color 0.15s var(--transition-easing-cubic);
    }

    &::after {
      margin-top: 2px;
      content: '';
      flex: 0 0 12px;
      display: block;
      width: 12px;
      top: 0;
      left: 0;
      transform: translate3d(4px, 4px, 0);
      height: 12px;
      border-radius: 12px;
      position: absolute;
      background-color: theme(--color-text-5);
      transition: background-color 0.15s var(--transition-easing-cubic),
        transform 0.15s var(--transition-easing-cubic);
    }
  }

  .toggle__input:disabled + .toggle {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .toggle__input:focus + .toggle:before {
    outline: 3px solid theme(--color-text-5-o10);
  }

  .toggle__input--touched:invalid + .toggle::before,
  .toggle--error::before {
    border-color: theme(--color-utility-warning);
  }

  .toggle__input--touched:invalid + .toggle::after,
  .toggle--error::after {
    background-color: theme(--color-utility-warning);
  }

  .toggle__input:checked + .toggle {
    &::before {
      border-color: theme(--color-primary);
    }

    &::after {
      background-color: theme(--color-primary);
      transform: translate3d(16px, 4px, 0);
    }
  }
</style>
