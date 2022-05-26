<script lang="ts">
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { user } from 'modules/authentication/store';
  import {
    isUserVerified,
    checkAndRedirect,
  } from 'modules/authorization/utils';
import { getUserInfo } from 'utils';

  $: {
    checkAndRedirect(getUserInfo(), 'public');
  }
</script>

{#if !isUserVerified(getUserInfo())}
  <slot />
{:else}
  <LoadingSpinner size="page" id="js-public-spinner" />
{/if}
