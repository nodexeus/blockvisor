<script type="ts">
  import Button from 'components/Button/Button.svelte';
  import ValueWithUtil from 'modules/forms/components/ValueWithUtil/ValueWithUtil.svelte';

  export let values = [];
  export let setStep;
  export let label = '';
  export let step;

  $: hasValues = Boolean(values.length);

  const switchToStep = () => {
    setStep(parseInt(step));
  };
</script>

{#if hasValues}
  <ul class="u-list-reset form-state">
    {#each values as item}
      <li>
        <ValueWithUtil id="network-type ">
          <svelte:fragment slot="label">{label}:{' '}</svelte:fragment>
          <strong slot="value">{item?.value ?? 'N/A'}</strong>
          <Button
            on:click={switchToStep}
            slot="util"
            size="small"
            style="outline">Change</Button
          >
        </ValueWithUtil>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .form-state {
    margin-bottom: 40px;
    display: flex;
    flex-direction: column;
    gap: ;
  }
</style>
