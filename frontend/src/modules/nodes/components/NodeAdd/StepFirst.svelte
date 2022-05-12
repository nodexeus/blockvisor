<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import TokenIcon from 'components/TokenIcon/TokenIcon.svelte';
  import CardSelector from 'modules/forms/components/CardSelector/CardSelector.svelte';
  import CardSelectorList from 'modules/forms/components/CardSelector/CardSelectorList.svelte';

  export let form;
  export let setStep;

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    $form.network.change(e.submitter.value);
    setStep(2);
  };
</script>

<div class="network">
  <CardSelectorList
    fieldName="network"
    id="node-network-list"
    {form}
    on:submit={handleSubmit}
  >
    <slot />
    <svelte:fragment slot="label">Select a network</svelte:fragment>
    <CardSelector>
      <svelte:fragment slot="label">
        <TokenIcon icon="hnt" />
        Helium<br />
        <small>HNT</small></svelte:fragment
      >
      <Button
        value="Helium"
        type="submit"
        slot="action"
        style="primary"
        size="small">Select</Button
      >
    </CardSelector>

    <CardSelector disabled>
      <span slot="top" class="t-tiny">Coming soon</span>
      <svelte:fragment slot="label">
        <TokenIcon icon="eth" />
        Ethereum 2.0<br />
        <small>ETH</small></svelte:fragment
      >
    </CardSelector>

    <CardSelector disabled>
      <span slot="top" class="t-tiny">Coming soon</span>
      <svelte:fragment slot="label">
        <TokenIcon icon="sol" />
        Solana<br />
        <small>SOL</small></svelte:fragment
      >
    </CardSelector>

    <CardSelector disabled>
      <span slot="top" class="t-tiny">Coming soon</span>
      <svelte:fragment slot="label">
        <TokenIcon icon="avax" />
        Avalanche<br />
        <small>AVAX</small></svelte:fragment
      >
    </CardSelector>

    <CardSelector disabled>
      <span slot="top" class="t-tiny">Coming soon</span>
      <svelte:fragment slot="label">
        <TokenIcon icon="algo" />
        Algorand<br />
        <small>ALGO</small></svelte:fragment
      >
    </CardSelector>
  </CardSelectorList>
</div>

<style>
  .network {
    & :global(svg) {
      color: theme(--color-text-2);
      margin-bottom: 6px;
    }

    small {
      @font tiny;
      color: theme(--color-text-2);
    }
  }
</style>
