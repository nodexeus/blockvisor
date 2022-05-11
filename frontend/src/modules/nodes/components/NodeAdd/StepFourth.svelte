<script lang="ts">
  import axios from 'axios';
  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { onMount } from 'svelte';

  let install_cmd;
  let host_id;
  let retrying;
  let claimed_at;
  let claimed_host_id;

  onMount(async () => {
    const res = await axios.post('/api/nodes/provisionNewHost', {
      org_id: '24f00a6c-1cb6-4660-8670-a9a7466699b2',
    });

    if (res.request.statusText === 'OK') {
      host_id = res.data?.node?.id;
      install_cmd = res.data?.node?.install_cmd;
    }
  });

  let isChecking = false;

  const checkProvision = (isRetrying?: boolean) => {
    isChecking = true;

    if (!host_id) {
      isChecking = false;
      return;
    }

    if (isRetrying) {
      retrying = true;

      setTimeout(() => {
        retrying = false;
      }, 2000);
    }

    axios
      .get('/api/nodes/confirmProvision', { params: { host_id } })
      .then((res) => {
        if (res.request.statusText === 'OK') {
          if (!res.data.host_id) {
            setTimeout(() => {
              checkProvision(true);
            }, 5000);
          }
        }
      });
  };

  const handleCheck = () => {
    checkProvision();
  };
</script>

<section class="provision-host">
  <p>Please execute this command on your host.</p>
  <p>It is a one time only command, and it will expire after 24h.</p>

  <pre class="code-block">{install_cmd || 'Loading command...'}
</pre>

  {#if install_cmd}
    <p class="continue-text">
      Click continue after you have run the command on your host.
      {#if retrying}
        <span class="retrying t-small">Host not found, retrying...</span>
      {/if}
    </p>

    <Button
      size="medium"
      display="block"
      style="primary"
      cssCustom="class"
      on:click={handleCheck}
    >
      {#if isChecking}
        &nbsp;
        <LoadingSpinner size="button" id="js-form-submit" />
      {:else}
        Continue
      {/if}</Button
    >

    <div class="cancel-button">
      {#if isChecking}
        <Button
          size="medium"
          display="block"
          style="warning"
          on:click={handleCheck}
        >
          Cancel
        </Button>
      {/if}
    </div>
  {/if}
</section>

<style>
  .provision-host {
    & :global(button) {
      position: relative;
    }
  }

  .code-block {
    margin: 18px 0;
    background: rgba(0, 0, 0, 0.5);
    padding: 10px 18px;
    white-space: break-spaces;
    word-break: break-all;
  }

  .continue-text {
    margin-bottom: 12px;
  }

  .cancel-button {
    margin-top: 12px;
  }

  .retrying {
    margin-top: 12px;
    opacity: 0.6;
  }
</style>
