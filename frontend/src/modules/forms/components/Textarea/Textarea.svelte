<script lang="ts">
  import { isObjectEmpty } from 'utils/isObjectEmpty';
  import { afterUpdate } from 'svelte';
  import { validators } from 'svelte-use-form';

  import InputHints from '../InputHints/InputHints.svelte';
  import InputLabel from '../InputLabel/InputLabel.svelte';
  import type { TextareaProps } from './Textarea.svelte';

  export let size: TextareaProps['size'] = 'medium';
  export let validate: TextareaProps['validate'] = [];
  export let labelClass: TextareaProps['labelClass'] = '';
  export let description: TextareaProps['description'] = '';
  export let field: TextareaProps['field'];

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field.touched || isObjectEmpty(field.errors));

  afterUpdate(checkIfValid);

  $: fieldClasses = [
    'textarea__field',
    `textarea__field--${size}`,
    `textarea__field--${isValid ? 'valid' : 'error'}`,
  ].join(' ');
</script>

<InputLabel name={$$props.name} disabled={$$props.disabled} {size} {labelClass}>
  <slot name="label" />
</InputLabel>

<div class="textarea__wrapper">
  <textarea
    use:validators={validate}
    class={fieldClasses}
    class:textarea__field--is-disabled={$$props.disabled}
    id={$$props.name}
    {...$$restProps}
  />
</div>

<InputHints
  name={$$props.name}
  disabled={$$props.disabled}
  {description}
  {isValid}
>
  <slot name="hints" />
</InputHints>

<style>
  .textarea__wrapper {
    position: relative;
    color: theme(--color-text-3);
  }

  .textarea__field {
    display: block;
    background-color: theme(--color-input-background);
    border-radius: 4px;
    font-weight: var(--font-weight-normal);
    color: inherit;
    padding: 12px 8px 12px 12px;
    border: 1px solid theme(--color-text-5-o10);
    transition: color 0.18s var(--transition-easing-cubic),
      border-color 0.2s var(--transition-easing-cubic);
    border: 1px solid theme(--color-input-background);
  }

  .textarea__field:focus {
    color: theme(--color-text-5);
    outline: 0;
    border-color: theme(--color-text-5-o30);
  }

  .textarea__field--error {
    border: 1px solid theme(--color-utility-warning);
  }

  .textarea__label {
    display: inline-block;
    color: theme(--color-text-5);
    margin-bottom: 4px;
  }

  .textarea__label--small {
    @mixin font tiny;
  }

  .textarea__label--medium {
    @mixin font small;
  }

  .textarea__label--large {
    @mixin font base;
  }

  .textarea__field--small {
    @mixin font small;
  }

  .textarea__field--medium {
    @mixin font small;
  }

  .textarea__field--large {
    @mixin font base;
  }

  .textarea__label--is-disabled,
  .textarea__field--is-disabled {
    opacity: 0.4;
    cursor: not-allowed;
    user-select: none;
  }
</style>
