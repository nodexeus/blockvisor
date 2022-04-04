<script lang="ts">
  import { goto } from '$app/navigation';
  import { FEATURE_FLAGS } from 'consts/featureFlags';
  import { ROUTES } from 'consts/routes';
  import { onMount } from 'svelte';

  onMount(() => {
    goto(FEATURE_FLAGS.BLOCKVISOR ? ROUTES.DASHBOARD : ROUTES.BROADCASTS);
  });
</script>
