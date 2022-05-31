<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { ENDPOINTS } from 'consts/endpoints';
  import { provisionedHostId } from 'modules/hosts/store/hostsStore';
  import { installedNode } from 'modules/nodes/store/nodesStore';
  import { httpClient } from 'utils/httpClient';

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

    let realNodeType;
    const nodeType = $form.nodeType.value;
    const hostId = $provisionedHostId;

    switch (nodeType) {
      case 'ETL':
        realNodeType = 'etl';
        break;
      case 'Node/api':
        realNodeType = 'node';
        break;
      case 'Validator':
        realNodeType = 'validator';
        break;
    }

    httpClient
      .post(ENDPOINTS.NODES.CREATE_NODE_POST, {
        org_id: '24f00a6c-1cb6-4660-8670-a9a7466699b2',
        host_id: hostId,
        chain_type: 'solana',
        node_type: realNodeType,
        status: 'installing',
        is_online: false,
      })
      .then((res) => {
        if (res.status === 200) {
          installedNode.set(res.data.node);
          success = true;
          installing = false;
        }
      })
      .catch((err) => console.log(err));

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
