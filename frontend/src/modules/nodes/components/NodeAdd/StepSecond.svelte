<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import CardSelector from 'modules/forms/components/CardSelector/CardSelector.svelte';
  import CardSelectorList from 'modules/forms/components/CardSelector/CardSelectorList.svelte';
  import {
    blockchains,
    selectedValidator,
  } from 'modules/nodes/store/nodesStore';
  import FormState from './FormState.svelte';

  export let form;
  export let setStep;

  const thisBlockchain = $blockchains.find(
    (item) => item.name === $form.network.value,
  );

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    $form.nodeType.change(e.submitter.value);
    setStep(3);
  };

  const nodeTypes = [];
  if (thisBlockchain.supports_node) {
    nodeTypes.push('Node/api');
  }
  if (thisBlockchain.supports_staking) {
    nodeTypes.push('Validator');
  }
  if (thisBlockchain.supports_etl) {
    nodeTypes.push('ETL');
  }
</script>

<FormState label="Network" values={[$form.network]} step={1} {setStep} />

<CardSelectorList
  fieldName="nodeType"
  id="node-network-list"
  {form}
  on:submit={handleSubmit}
>
  <slot />
  <svelte:fragment slot="label">Select node type</svelte:fragment>

  {#each nodeTypes as item, i}
    <CardSelector index={i}>
      <svelte:fragment slot="label">{item}</svelte:fragment>
      <Button
        value={item}
        type="submit"
        slot="action"
        style="primary"
        size="small">Select</Button
      >
    </CardSelector>
  {/each}
</CardSelectorList>
