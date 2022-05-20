<script lang="ts">
  import axios from 'axios';

  import Button from 'components/Button/Button.svelte';
  import TokenIcon from 'components/TokenIcon/TokenIcon.svelte';
  import CardSelector from 'modules/forms/components/CardSelector/CardSelector.svelte';
  import CardSelectorList from 'modules/forms/components/CardSelector/CardSelectorList.svelte';
  import { blockchains } from 'modules/nodes/store/nodesStore';
  import { onMount } from 'svelte';

  export let form;
  export let setStep;

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    $form.network.change(e.submitter.value);
    setStep(2);
  };

  onMount(() => {
    axios.get('/api/nodes/getBlockchains').then((res) => {
      if (res.statusText === 'OK') {
        const active = res.data.filter(
          (item: Blockchain) => item.status === 'production',
        );
        const inactive = res.data.filter(
          (item: Blockchain) => item.status !== 'production',
        );

        blockchains.set([...active, ...inactive]);
      }
    });
  });
</script>

<div class="network">
  <CardSelectorList id="node-network-list" {form} on:submit={handleSubmit}>
    <slot />
    <svelte:fragment slot="label">Select a network</svelte:fragment>

    {#each $blockchains as item, i}
      <CardSelector disabled={item.status !== 'production'} index={i}>
        <svelte:fragment slot="label">
          {#if item.token}
            <TokenIcon icon={item.token.toLowerCase()} />
          {/if}
          {item.name}<br />
          <small>{item.token || ''}</small></svelte:fragment
        >
        <!-- {#if item.status === 'production'} -->
        <Button
          value={item.name}
          type="submit"
          slot="action"
          style="primary"
          size="small">Select</Button
        >
        <!-- {/if} -->
      </CardSelector>
    {/each}
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
