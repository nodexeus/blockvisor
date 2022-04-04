<script lang="ts">
  import SubscriptionPlans from '../SubscriptionPlans/SubscriptionPlans.svelte';

  export let hasSubscription = false;
</script>

{#if hasSubscription}
  <slot />
{:else}
  <SubscriptionPlans />
{/if}
