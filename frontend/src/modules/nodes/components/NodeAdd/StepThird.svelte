<script lang="ts">
  import FormState from './FormState.svelte';
  import Button from 'components/Button/Button.svelte';
  import TextWithActionTop from 'modules/dashboard/components/TextWithAction/TextWithActionTop.svelte';
  import FileUpload from 'modules/forms/components/FileUpload/FileUpload.svelte';
  import NewValidatorForm from './NewValidatorForm.svelte';

  export let handleFilesRemove;
  export let handleFilesSelect;
  export let swarmKey = { accepted: [], rejected: [] };
  export let form;
  export let setStep;

  let shouldCreateNewValidator = false;

  const handleToggleCreate = () => {
    shouldCreateNewValidator = !shouldCreateNewValidator;
  };

  $: hasAcceptedFiles = Boolean(swarmKey.accepted.length);

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();
    setStep(4);
  };

  const handleExistingValidator = () => {
    if (!hasAcceptedFiles) {
      return;
    }

    setStep(4);
  };
</script>

<FormState label="Node type" values={[$form.nodeType]} step={2} {setStep} />

<slot />

{#if shouldCreateNewValidator}
  <NewValidatorForm {setStep} {form} />
{:else}
  <section class="step-validator">
    <TextWithActionTop>
      <h2 class="t-large" slot="title">Create a New Validator</h2>
      <div class="step-validator__content">
        Nullam pulvinar, metus ut bibendum sagittis, elit massa lacinia elit.
      </div>
      <Button
        on:click={handleToggleCreate}
        slot="action"
        size="small"
        style="secondary">Create New</Button
      >
    </TextWithActionTop>

    <TextWithActionTop>
      <h2 class="t-large" slot="title">Import an Existing Validator</h2>
      <div class="step-validator__content">
        Nullam pulvinar, metus ut bibendum sagittis, elit massa lacinia elit.

        <div class="step-validator__input">
          <FileUpload
            files={swarmKey.accepted}
            on:click={handleFilesRemove}
            on:drop={handleFilesSelect}
            >Upload an existing swarm key
          </FileUpload>
        </div>
      </div>
      <Button
        on:click={handleExistingValidator}
        disabled={!hasAcceptedFiles}
        slot="action"
        size="small"
        style="secondary">Select</Button
      >
    </TextWithActionTop>
  </section>
{/if}

<style>
  .step-validator {
    margin-top: 60px;

    &__input {
      margin-top: 24px;
    }
  }
</style>
