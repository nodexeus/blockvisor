<script lang="ts">
  import LayoutTwoColumn from 'components/LayoutTwoColumn/LayoutTwoColumn.svelte';
  import { pageTransition } from 'consts/animations';
  import { APPS } from 'models/App';
  import SettingsNav from 'modules/admin-console/components/SettingsNav/SettingsNav.svelte';
  import { app } from 'modules/app/store';
  import BlockVisorFeature from 'modules/feature-flags/components/BlockVisorFeature/BlockVisorFeature.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    app.setActiveApp(APPS.BLOCKVISOR);
  });
</script>

<BlockVisorFeature>
  <LayoutTwoColumn transition={pageTransition}>
    <SettingsNav slot="sidebar" />
    <slot />
  </LayoutTwoColumn>
</BlockVisorFeature>
