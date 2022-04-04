<script lang="ts">
  import { validators } from 'svelte-use-form';
  import Svelecte from 'svelecte';
  import { afterUpdate } from 'svelte';
  import InputHints from '../InputHints/InputHints.svelte';
  import { isObjectEmpty } from 'utils';

  export let validate = [];
  export let field;
  export let name;
  export let description = '';
  export let options = [];

  $: classes = [
    'multiselect',
    `multiselect--${isValid ? 'valid' : 'error'}`,
  ].join(' ');

  $: isValid = true;

  const checkIfValid = () =>
    (isValid = !field.touched || isObjectEmpty(field.errors));

  afterUpdate(() => {
    checkIfValid();
  });

  export let labelAsValue = false;

  let selection = [];
  let value = [];

  const handleChange = () => {
    field.touched = true;
    const newValue = value.join(',');
    field.change(newValue);
  };

  const handleBlur = () => {
    field.touched = true;
  };
</script>

{#if $$slots.label}
  <label class="multiselect-label" for={name}>
    <slot name="label" />
  </label>
{/if}

<input
  {name}
  use:validators={validate}
  tabindex="-1"
  type="hidden"
  class="visually-hidden"
/>

<Svelecte
  id={name}
  on:blur={handleBlur}
  on:change={handleChange}
  class={classes}
  {options}
  {labelAsValue}
  bind:readSelection={selection}
  bind:value
  multiple
  {...$$restProps}
/>

{#if $$slots.hints || description}
  <InputHints {name} disabled={false} {description} {isValid}>
    <slot name="hints" />
  </InputHints>
{/if}

<style global>
  .svelecte.multiselect {
    --sv-icon-color: theme(--color-border-4);
    --sv-min-height: 60px;
    --sv-border-color: transparent;
    --sv-active-border: theme(--color-text-4);
    --sv-color: theme(--color-text-4);
    --sv-bg: theme(--color-input-background);

    &--error {
      .sv-control {
        border: 1px solid theme(--color-utility-warning);
      }
    }

    & .sv-control {
      padding: 16px 12px;
      gap: 16px;

      & > .icon-slot {
        display: none;
      }
    }

    & .sv-dropdown {
      border-width: 0;
      margin-top: -1px;
      border-radius: 0;
      background-color: theme(--color-overlay-background-1);

      & .highlight {
        background-color: theme(--color-primary);
        color: theme(--color-text-1);
      }

      &-scroll {
        padding: 0;
        max-height: 250px;
      }

      & .sv-item-content {
        padding: 8px 12px;
      }

      & .sv-dd-item {
        cursor: pointer;

        &-active,
        &:hover,
        &:active,
        &:focus {
          background-color: theme(--color-text-5-o3);
        }
      }
    }

    & .sv-content {
      padding: 0;

      & .sv-item {
        background-color: theme(--color-text-5-o10);
        border-radius: 4px;
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 4px 16px;
        color: theme(--color-text-4);

        &-btn {
          cursor: pointer;
          padding: 0;
          opacity: 0.3;
        }
      }
    }

    & .indicator-container {
      padding: 0;
    }

    & .sv-input-row {
      gap: 8px;
    }
  }

  .multiselect-label {
    color: theme(--color-text-3);
    margin-bottom: 4px;
  }
</style>
