<script lang="ts">
  import type { InputLabelProps } from './InputLabel.svelte';

  export let name: InputLabelProps['name'];
  export let labelClass: InputLabelProps['labelClass'] = '';
  export let size: InputLabelProps['size'] = 'medium';
  export let disabled: InputLabelProps['disabled'] = false;

  const labelClasses = ['input-label', `input-label--${size}`, labelClass].join(
    ' ',
  );
</script>

<label
  for={name}
  class={labelClasses}
  class:textarea__label--is-disabled={disabled}
>
  <slot />
</label>

<style>
  .input-label {
    display: inline-block;
    color: theme(--color-text-3);
    margin-bottom: 4px;

    &--small {
      @mixin font tiny;
    }

    &--medium {
      @mixin font small;
    }

    &--large {
      @mixin font base;
    }

    &--is-disabled {
      opacity: 0.4;
      cursor: not-allowed;
      user-select: none;
    }
  }
</style>
