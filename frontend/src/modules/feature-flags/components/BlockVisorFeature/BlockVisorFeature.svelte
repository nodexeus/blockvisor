<script>
  import { FEATURE_FLAGS } from 'consts/featureFlags';
  import BlockVisorFallback from '../Fallbacks/BlockVisorFallback.svelte';
  import BlockVisorFallbackSmall from '../Fallbacks/BlockVisorFallbackSmall.svelte';

  import FeatureFlagWrapper from '../FeatureFlagWrapper/FeatureFlagWrapper.svelte';
  export let type = 'overlay';
</script>

<FeatureFlagWrapper shouldDisplay={FEATURE_FLAGS.BLOCKVISOR}>
  <slot />
  <svelte:fragment slot="fallback">
    {#if type === 'overlay'}
      <BlockVisorFallback />
    {:else}
      <BlockVisorFallbackSmall />
    {/if}
  </svelte:fragment>
</FeatureFlagWrapper>
