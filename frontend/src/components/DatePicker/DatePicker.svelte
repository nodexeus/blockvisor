<script>
  import { DatePicker } from 'date-picker-svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import IconCalendar from 'icons/calendar-12.svg';
  import IconCaret from 'icons/caret-micro.svg';
  import { format } from 'date-fns';
  import { flyDefault } from 'consts/animations';
  import { fly } from 'svelte/transition';
  import { clickOutside } from 'utils';
  export let value;
  export let dropdownActive = false;
</script>

<div
  class="datepicker-wrapper"
  use:clickOutside
  on:click_outside={() => (dropdownActive = false)}
>
  <div class="datepicker-input-wrapper">
    <Input
      value={format(value, 'MM/dd/yyyy HH:mm')}
      size="small"
      name="start"
      on:focus={() => (dropdownActive = !dropdownActive)}
    >
      <span slot="utilLeft"><IconCalendar /></span>
      <span
        on:click={() => (dropdownActive = !dropdownActive)}
        slot="utilRight"
        class="input-caret"><IconCaret /></span
      >
    </Input>
  </div>
  {#if dropdownActive}
    <div transition:fly|local={flyDefault}>
      <DatePicker bind:value />
    </div>
  {/if}
</div>

<style>
  :global(body) {
    --date-picker-foreground: #f8faf6;
    --date-picker-background: #363938;
    --date-picker-highlight-border: #bff589;
    --date-picker-selected-color: #212423;
    --date-picker-selected-background: #bff589;
  }

  .datepicker-wrapper {
    max-width: 244px;
  }

  .datepicker-input-wrapper {
    margin-bottom: 8px;
    margin-left: auto;
    max-width: 200px;
  }

  .input-caret {
    cursor: pointer;
  }
</style>
