<script lang="ts">
  import { HintGroup } from 'svelte-use-form';
  import type { InputHintProps } from './InputHints.svelte';

  export let isValid: InputHintProps['isValid'] = true;
  export let description: InputHintProps['description'] = '';
  export let disabled: InputHintProps['disabled'] = false;
  export let name: InputHintProps['name'];
</script>

{#if isValid && description}
  <div
    class="input-hints input-hints--description"
    class:input-hints--is-disabled={disabled}
  >
    {description}
  </div>
{/if}

{#if !isValid && $$slots}
  <div
    class="input-hints input-hints--warning"
    class:input-hints--is-disabled={disabled}
  >
    <HintGroup for={name}>
      <slot />
    </HintGroup>
  </div>
{/if}

<style>
  .input-hints {
    @mixin font small;
    margin-top: 4px;

    &--description {
      color: theme(--color-text-3);
    }

    &--warning {
      color: theme(--color-utility-warning);
    }

    &--is-disabled {
      opacity: 0.4;
      cursor: not-allowed;
      user-select: none;
    }
  }
</style>
