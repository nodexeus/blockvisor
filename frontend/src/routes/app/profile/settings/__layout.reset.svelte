<script lang="ts">
  import BaseLayout from 'modules/app/components/BaseLayout/BaseLayout.svelte';
  import Layout from 'modules/settings/components/Layout/Layout.svelte';
  import { onMount } from 'svelte';
  import { app } from 'modules/app/store';
  import { APPS } from 'models/App';

  onMount(() => {
    app.setActiveApp(APPS.BLOCKVISOR);
  });
</script>

<BaseLayout>
  <Layout>
    <slot />
  </Layout>
</BaseLayout>
