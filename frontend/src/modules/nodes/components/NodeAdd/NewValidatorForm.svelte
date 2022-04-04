<script lang="ts">
  import { fly } from 'svelte/transition';
  import Button from 'components/Button/Button.svelte';
  import { flyDefault } from 'consts/animations';
  import TextWithActionTop from 'modules/dashboard/components/TextWithAction/TextWithActionTop.svelte';
  import SelectWithDescription from 'modules/forms/components/SelectWithDescription/SelectWithDescription.svelte';

  export let form;
  export let setStep;

  const handleSelect = (e) => {
    const { value } = e.target;
    $form.host.change(value);
    setStep(4);
  };

  const handleNextStep = () => {
    setStep(4);
  };
</script>

<section in:fly|local={flyDefault} class="new-validator">
  <TextWithActionTop>
    <h2 class="t-large" slot="title">Blockjoy Host</h2>
    <div class="step-validator__content">
      Nullam pulvinar, metus ut bibendum sagittis, elit massa lacinia elit.
    </div>
    <Button
      on:click={handleNextStep}
      slot="action"
      size="small"
      style="secondary">Select</Button
    >
  </TextWithActionTop>

  <div class="new-validator__controls">
    <TextWithActionTop>
      <h2 class="t-large" slot="title">Select a Host</h2>
      <div class="step-validator__content">
        Nullam pulvinar, metus ut bibendum sagittis, elit massa lacinia elit.
      </div>
    </TextWithActionTop>

    <SelectWithDescription>
      <svelte:fragment slot="title">HostFox</svelte:fragment>
      <div class="new-validator__location" slot="description">
        <span> 212.213.214.2 </span>
        <span> Zagreb, Croatia </span>
      </div>
      <Button
        type="button"
        slot="action"
        size="small"
        style="outline"
        value="some-id-1"
        on:click={handleSelect}>Select</Button
      >
    </SelectWithDescription>

    <SelectWithDescription>
      <svelte:fragment slot="title">HostFox</svelte:fragment>
      <div class="new-validator__location" slot="description">
        <span> 212.213.214.2 </span>
        <span> Zagreb, Croatia </span>
      </div>
      <Button
        type="button"
        slot="action"
        size="small"
        style="outline"
        value="some-id-2"
        on:click={handleSelect}>Select</Button
      >
    </SelectWithDescription>
  </div>

  <TextWithActionTop>
    <h2 class="t-large" slot="title">Use My Own Host</h2>
    <div class="step-validator__content">
      Nullam pulvinar, metus ut bibendum sagittis, elit massa lacinia elit.
    </div>
    <Button
      on:click={handleNextStep}
      slot="action"
      size="small"
      style="secondary">Select</Button
    >
  </TextWithActionTop>
</section>

<style>
  .new-validator {
    border-top: 1px solid theme(--color-border-2);
    padding-top: 20px;

    &__location {
      display: flex;
      gap: 12px;
    }

    & :global(.text-with-action-top) {
      margin-bottom: 0;
      padding-bottom: 60px;
    }

    &__controls {
      border-top: 1px solid theme(--color-border-2);
      border-bottom: 1px solid theme(--color-border-2);
      padding-top: 20px;
      padding-bottom: 60px;
      margin-bottom: 20px;

      & :global(.text-with-action-top) {
        margin-bottom: 0;
        padding-bottom: 30px;
      }
    }
  }
</style>
