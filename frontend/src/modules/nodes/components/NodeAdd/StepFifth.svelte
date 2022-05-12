<script lang="ts">
  import axios from 'axios';

  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { provisionedHostId } from 'modules/hosts/store/hostsStore';

  export let setStep;
  export let form;

  let installing;

  const handleNextStep = () => {
    setStep(6);
  };

  async function handleInstall() {
    installing = true;

    const res = await axios.post('/api/nodes/installNode', {
      host_id: $provisionedHostId,
      nodeType: $form.nodeType.value,
    });

    console.log(res);

    setTimeout(() => {
      installing = false;
    }, 1000);
  }
</script>

<section class="installing-node">
  <p class="installing-node__text">
    Installing node type<strong> {$form.nodeType.value}</strong>. Click to
    install:
  </p>

  <Button
    size="medium"
    display="block"
    style="primary"
    on:click={handleInstall}
  >
    {#if installing}
      &nbsp;
      <LoadingSpinner size="button" id="js-form-submit" />
    {:else}
      Continue
    {/if}
  </Button>
</section>

<style>
  .installing-node {
    padding-bottom: 100px;
    & :global(button) {
      position: relative;
    }
  }

  .installing-node__text {
    margin-bottom: 18px;
  }
</style>
