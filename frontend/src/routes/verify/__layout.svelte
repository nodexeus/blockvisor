<script lang="ts">
  import { goto } from '$app/navigation';

  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { FEATURE_FLAGS } from 'consts/featureFlags';
  import { ROUTES } from 'consts/routes';
  import Layout from 'modules/authentication/components/Layout/Layout.svelte';
  import { user } from 'modules/authentication/store';
  import { onMount } from 'svelte';

  $: isLoggedIn = !!$user;
  $: isVerified = isLoggedIn && $user?.verified;

  const checkUserVerification = () => {
    if (!$user) {
      goto(ROUTES.LOGIN, { replaceState: true });
      return;
    }

    if (isVerified) {
      goto(FEATURE_FLAGS.BLOCKVISOR ? ROUTES.DASHBOARD : ROUTES.BROADCASTS, {
        replaceState: true,
      });
    }
  };

  onMount(checkUserVerification);

  $: () => {
    checkUserVerification();
  };
</script>

{#if isVerified || !isLoggedIn}
  <LoadingSpinner size="page" id="js-loading-verify" />
{:else}
  <Layout title="We have Sent a Link to Your Email Address.">
    <slot />
  </Layout>
{/if}
