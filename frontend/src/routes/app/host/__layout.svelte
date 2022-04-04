<script>
  import { APPS } from 'models/App';
  import { app } from 'modules/app/store';
  import BlockVisorFeature from 'modules/feature-flags/components/BlockVisorFeature/BlockVisorFeature.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    app.setActiveApp(APPS.BLOCKVISOR);
  });
</script>

<BlockVisorFeature>
  <slot />
</BlockVisorFeature>
