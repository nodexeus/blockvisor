<script lang="ts">
  import { isObjectEmpty } from 'utils/isObjectEmpty';
  import { afterUpdate } from 'svelte';

  import { validators } from 'svelte-use-form';
  import InputLabel from '../InputLabel/InputLabel.svelte';
  import InputUtil from '../InputUtil/InputUtil.svelte';

  import type { SelectProps } from './Select.svelte';
  import InputHints from '../InputHints/InputHints.svelte';

  export let size: SelectProps['size'] = 'medium';
  export let style: SelectProps['style'] = 'default';
  export let items: SelectProps['items'];
  export let validate: SelectProps['validate'] = [];
  export let labelClass: SelectProps['labelClass'] = '';
  export let inputClass: SelectProps['inputClass'] = '';
  export let description: SelectProps['description'] = '';
  export let field: SelectProps['field'];

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field?.touched || isObjectEmpty(field.errors));

  afterUpdate(checkIfValid);

  $: fieldClasses = [
    'input__field',
    'input__field--arrow',
    `input__field--${size}`,
    `input__field--${style}`,
    `input__field--${isValid ? 'valid' : 'error'}`,
    inputClass,
  ].join(' ');
</script>

<InputLabel {size} name={$$props.name} {labelClass} disabled={$$props.disabled}>
  <slot name="label" />{#if $$restProps.required}*{/if}
</InputLabel>

<div class="input__wrapper">
  {#if $$slots.utilLeft}
    <InputUtil position="left">
      <slot name="utilLeft" />
    </InputUtil>
  {/if}
  <select
    use:validators={validate}
    class={fieldClasses}
    class:input__field--with-util-left={$$slots.utilLeft}
    class:input__field--is-disabled={$$props.disabled}
    id={$$props.name}
    {...$$restProps}
  >
    {#each items as item}
      <option value={item?.value}>{item?.label}</option>
    {/each}
  </select>
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
  .input__wrapper {
    position: relative;
    color: theme(--color-text-3);
  }

  .input__field {
    border-radius: 4px;
    color: inherit;
    cursor: pointer;
    transition: color 0.18s var(--transition-easing-cubic),
      border-color 0.2s var(--transition-easing-cubic);
  }

  .input__field--default {
    background-color: theme(--color-input-background);
    font-weight: var(--font-weight-normal);
    border: 1px solid theme(--color-input-background);
  }

  .input__field--outline {
    border: 1px solid theme(--color-text-2);
    border-radius: 4px;
    color: theme(--color-text-5);
    font-weight: var(--font-weight-bold);
  }

  .input__field--arrow {
    background-size: 30px;
    background-repeat: no-repeat;
    background-position: right 12px center;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 16 16'><path stroke='%23F8FAF6' d='m5 7 2.828 2.828L10.657 7'/></svg>");
  }

  .input__field:focus {
    color: theme(--color-text-5);
  }

  .input__field--error {
    border-color: theme(--color-utility-warning);
  }

  .input__field--small {
    @mixin font small;
    padding: 6px 48px 6px 12px;
  }

  .input__field--medium {
    @mixin font small;
    padding: 10px 48px 10px 12px;
  }

  .input__field--large {
    @mixin font base;
    padding: 16px 48px 16px 12px;
  }

  .input__field--with-util-left {
    padding-left: 36px;
  }

  .input__util {
    color: theme(--color-text-3);
    fill: theme(--color-text-3);
    position: absolute;
    top: 50%;
    transform: translate3d(0, -50%, 0);
  }

  .input__util--left {
    left: 12px;
  }

  .input__util :global(svg path) {
    fill: currentColor;
  }

  .input__hints {
    @mixin font small;
    margin-top: 4px;
  }

  .input__hints--description {
    color: theme(--color-text-3);
  }

  .input__hints--warning {
    color: theme(--color-utility-warning);
  }

  .input__field--is-disabled,
  .input__hints--is-disabled {
    opacity: 0.4;
    cursor: not-allowed;
    user-select: none;
  }
</style>
