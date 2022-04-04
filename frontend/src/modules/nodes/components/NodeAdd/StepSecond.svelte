<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import CardSelector from 'modules/forms/components/CardSelector/CardSelector.svelte';
  import CardSelectorList from 'modules/forms/components/CardSelector/CardSelectorList.svelte';
  import FormState from './FormState.svelte';

  export let form;
  export let setStep;

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    $form.nodeType.change(e.submitter.value);
    setStep(3);
  };
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
  <CardSelector>
    <svelte:fragment slot="label">Node/api</svelte:fragment>
    <Button
      value="Node/api"
      type="submit"
      slot="action"
      style="primary"
      size="small">Select</Button
    >
  </CardSelector>

  <CardSelector>
    <svelte:fragment slot="label">Validator</svelte:fragment>
    <Button
      value="Validator"
      type="submit"
      slot="action"
      style="primary"
      size="small">Select</Button
    >
  </CardSelector>

  <CardSelector>
    <svelte:fragment slot="label">ETL</svelte:fragment>
    <Button value="ETL" type="submit" slot="action" style="primary" size="small"
      >Select</Button
    >
  </CardSelector>
</CardSelectorList>
