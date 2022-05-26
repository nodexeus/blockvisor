<script lang="ts">
  import { clickOutside } from 'utils';
  import Button from 'components/Button/Button.svelte';
  import Dropdown from 'components/Dropdown/Dropdown.svelte';
  import IconButton from 'components/IconButton/IconButton.svelte';

  export let buttonProps = {};
  export let iconButton = false;
  export let position = 'left';

  let isActive = false;

  const handleClickOutside = () => {
    isActive = false;
  };

  const onDropdownToggle = () => {
    isActive = !isActive;
  };

  const classes = [`dropdown-action`, `dropdown-action--${position}`].join(' ');
</script>

<div class={classes} use:clickOutside on:click_outside={handleClickOutside}>
  {#if iconButton}
    <IconButton
      on:click={onDropdownToggle}
      style="outline"
      size="small"
      {...buttonProps}
    >
      <slot name="label" />
    </IconButton>
  {:else}
    <Button
      on:click={onDropdownToggle}
      style="outline"
      size="small"
      {...buttonProps}
    >
      <slot name="label" />
    </Button>
  {/if}

  <Dropdown {isActive}>
    <slot name="content" />
  </Dropdown>
</div>

<style>
  .dropdown-action {
    position: relative;
    display: inline-block;

    &--right {
      & :global(.dropdown) {
        left: auto;
        right: 0;
      }
    }

    & :global(.dropdown) {
      margin-top: 10px;
    }
  }
</style>
