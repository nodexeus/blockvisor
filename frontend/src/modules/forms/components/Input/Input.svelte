<script lang="ts">
  import { isObjectEmpty } from 'utils';
  import { afterUpdate } from 'svelte';
  import { validators } from 'svelte-use-form';

  import InputHints from '../InputHints/InputHints.svelte';
  import InputLabel from '../InputLabel/InputLabel.svelte';
  import InputUtil from '../InputUtil/InputUtil.svelte';
  import type { InputProps } from './Input.svelte';

  export let size: InputProps['size'] = 'medium';
  export let style = 'default';
  export let validate: InputProps['validate'] = [];
  export let labelClass: InputProps['labelClass'] = '';
  export let description: InputProps['description'] = '';
  export let field: InputProps['field'];

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field?.touched || isObjectEmpty(field?.errors));

  afterUpdate(checkIfValid);

  $: fieldClasses = [
    'input__field',
    `input__field--${style}`,
    `input__field--${size}`,
    `input__field--${isValid ? 'valid' : 'error'}`,
  ].join(' ');
</script>

<InputLabel name={$$props.name} disabled={$$props.disabled} {size} {labelClass}>
  <slot name="label" />{#if $$restProps.required}*{/if}
</InputLabel>

<div class="input__wrapper">
  {#if $$slots.utilLeft}
    <InputUtil position="left">
      <slot name="utilLeft" />
    </InputUtil>
  {/if}

  <input
    on:keyup
    on:focus
    use:validators={validate}
    class={fieldClasses}
    class:input__field--with-util-left={$$slots.utilLeft}
    class:input__field--with-util-right={$$slots.utilRight}
    class:input__field--is-disabled={$$props.disabled}
    id={$$props.name}
    {...$$restProps}
  />

  {#if $$slots.utilRight}
    <InputUtil position="right">
      <slot name="utilRight" />
    </InputUtil>
  {/if}
</div>

{#if $$slots.hints || description}
  <InputHints
    name={$$props.name}
    disabled={$$props.disabled}
    {description}
    {isValid}
  >
    <slot name="hints" />
  </InputHints>
{/if}

<style>
  .input__wrapper {
    position: relative;
    color: theme(--color-text-3);
  }

  .input__field {
    border-radius: 4px;
    font-weight: var(--font-weight-normal);
    color: inherit;
    transition: color 0.18s var(--transition-easing-cubic),
      background-color 0.18s var(--transition-easing-cubic),
      border-color 0.18s var(--transition-easing-cubic);

    &--default {
      border: 1px solid theme(--color-text-5-o10);
      background-color: theme(--color-input-background);
    }

    &--outline {
      background-color: transparent;
      border: 1px solid theme(--color-text-5-o10);

      &:hover {
        border-color: theme(--color-text-2);
      }

      &:focus {
        border-color: theme(--color-text-4);
        outline: 0;
      }
    }
  }

  .input__field:focus {
    outline: 0;
    border-color: theme(--color-text-5-o30);
    color: theme(--color-text-5);
  }

  .input__field--error {
    border-color: theme(--color-utility-warning);
  }

  .input__field--small {
    @mixin font small;
    padding: 5px 12px;
  }

  .input__field--medium {
    @mixin font small;
    padding: 9px 12px;
  }

  .input__field--large {
    @mixin font base;
    padding: 15px 12px;
  }

  .input__field--with-util-left {
    padding-left: 36px;
  }

  .input__field--with-util-right {
    padding-right: 36px;
  }

  .input__field--is-disabled {
    opacity: 0.4;
    cursor: not-allowed;
    user-select: none;
  }
</style>
