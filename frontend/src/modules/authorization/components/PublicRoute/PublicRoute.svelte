<script lang="ts">
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { user } from 'modules/authentication/store';
  import {
    isUserVerified,
    checkAndRedirect,
  } from 'modules/authorization/utils';

  $: {
    checkAndRedirect($user, 'public');
  }
</script>

{#if !isUserVerified($user)}
  <slot />
{:else}
  <LoadingSpinner size="page" id="js-public-spinner" />
{/if}
