<script lang="ts">
  import { isObjectEmpty } from 'utils';
  import { afterUpdate } from 'svelte';
  import { validators } from 'svelte-use-form';

  import InputHints from '../InputHints/InputHints.svelte';
  import InputLabel from '../InputLabel/InputLabel.svelte';
  import Pill from 'components/Pill/Pill.svelte';
  import PillBox from 'components/PillBox/PillBox.svelte';

  export let size = 'large';
  export let validate = [];
  export let labelClass = '';
  export let description = '';
  export let field;

  export let limit = 3;

  let inputField;

  let currentValue = '';
  let submittedValues = $$restProps?.value ? $$restProps?.value.split(',') : [];

  const handleClick = () => {
    inputField?.focus();
  };

  $: isEnabled = submittedValues.length < limit;

  const handleDelete = (e) => {
    const { pillValue } = e.currentTarget.dataset;
    const filteredValues = submittedValues.filter(
      (value) => value !== pillValue,
    );
    submittedValues = filteredValues;
    field.change(filteredValues.join(','));
  };

  const onInputBlur = () => {
    currentValue = '';
    inputField.value = '';
  };

  const onInputChange = (e) => {
    e.preventDefault();
    e.stopPropagation();

    const shouldSubmitTag =
      e.currentTarget.value.includes(',') || e.key === 'Enter';

    if (shouldSubmitTag && e.currentTarget.value.length <= 1) {
      currentValue = '';
      inputField.value = '';
      return;
    }

    if (shouldSubmitTag) {
      if (submittedValues.findIndex((value) => value === currentValue) > -1) {
        currentValue = '';
        inputField.value = '';
        return;
      }

      const newValue = [...submittedValues, currentValue];
      submittedValues = newValue;
      inputField.value = '';
      field.change(submittedValues.join(','));
      currentValue = '';
      setTimeout(() => {
        inputField.scrollIntoView({ inline: 'end', behavior: 'smooth' });
      }, 50);
    }

    currentValue = e.currentTarget.value;
  };

  const checkSubmit = (e) => {
    if (e.key !== 'Enter') {
      return;
    }

    e.preventDefault();
    e.stopPropagation();
    onInputChange(e);
  };

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field.touched || isObjectEmpty(field.errors));

  afterUpdate(checkIfValid);

  $: fieldClasses = ['visually-hidden', 'tagsfield__field'].join(' ');
</script>

<InputLabel name={$$props.name} disabled={$$props.disabled} {size} {labelClass}>
  <slot name="label" />
</InputLabel>

<div
  class:tagsfield--is-disabled={$$props.disabled}
  class={`tagsfield__wrapper tagsfield--${size} tagsfield--${
    isValid ? 'valid' : 'error'
  }`}
  on:click={handleClick}
>
  <PillBox>
    {#each submittedValues as value (value)}
      <li>
        <Pill data-pill-value={value} on:click={handleDelete}>{value}</Pill>
      </li>
    {/each}
  </PillBox>

  <input
    use:validators={validate}
    class={fieldClasses}
    tabindex="-1"
    {...$$restProps}
  />

  {#if isEnabled}
    <input
      on:blur={onInputBlur}
      on:keydown={checkSubmit}
      id={$$props.name}
      bind:this={inputField}
      on:input={onInputChange}
      bind:value={currentValue}
      placeholder={$$props.placeholder}
      class="tagsfield__field"
    />
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
  .tagsfield__wrapper {
    display: flex;
    gap: 8px;
    position: relative;
    color: theme(--color-text-3);
    background-color: theme(--color-input-background);
    border-radius: 4px;
    font-weight: var(--font-weight-normal);
    color: inherit;
    transition: color 0.18s var(--transition-easing-cubic),
      border-color 0.2s var(--transition-easing-cubic);
    border: 1px solid theme(--color-text-5-o10);
    overflow-x: auto;
    overflow-y: visible;
  }

  .tagsfield__field {
    opacity: 0.5;
    width: 15ch;
    background-color: transparent;
    border-width: 0;
    border-radius: 4px;
    transition: opacity 0.15s var(--transition-easing-cubic);
    outline: 0;
    flex-grow: 1;
  }

  .tagsfield__field:focus {
    opacity: 1;
    color: theme(--color-text-5);
  }

  .tagsfield__wrapper:focus-within {
    outline: 0;
    border-color: theme(--color-text-5-o30);
  }

  .tagsfield--error {
    border-color: theme(--color-utility-warning);
  }

  .tagsfield--small {
    @mixin font small;
    padding: 4px 12px;
  }

  .tagsfield--medium {
    @mixin font small;
    padding: 8px 12px;
  }

  .tagsfield--large {
    @mixin font base;
    padding: 15px 12px;
  }

  .tagsfield--is-disabled {
    opacity: 0.4;
    cursor: not-allowed;
    user-select: none;
  }
</style>
