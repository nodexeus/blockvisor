<script lang="ts">
  import axios from 'axios';

  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { provisionedHostId } from 'modules/hosts/store/hostsStore';
  import { installedNode } from 'modules/nodes/store/nodesStore';

  export let setStep;
  export let form;

  let success = false;
  let installing;

  const handleNextStep = () => {
    setStep(6);
  };

  async function handleInstall() {
    success = false;
    installing = true;

    console.log($provisionedHostId);
    console.log($form.nodeType.value);

    axios
      .get('/api/nodes/installNode', {
        params: {
          host_id: $provisionedHostId,
          node_type: $form.nodeType.value,
        },
      })
      .then((res) => {
        if (res.statusText === 'OK') {
          installedNode.set(res.data.node);
          success = true;
          installing = false;
        }
      });

    setTimeout(() => {
      installing = false;
    }, 1000);
  }
</script>

<section class="installing-node">
  <p class="installing-node__text">
    Installing node type<strong>{' '}{$form.nodeType.value}</strong>. Click to
    install:
  </p>

  <Button
    size="medium"
    disabled={installing}
    display="block"
    style="primary"
    on:click={success ? handleNextStep : handleInstall}
  >
    {#if installing && !success}
      &nbsp;
      <LoadingSpinner size="button" id="js-form-submit" />
    {/if}

    {#if !installing && !success}
      Continue
    {/if}

    {#if !installing && success}
      Review
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
