<script>
  import { ROUTES } from 'consts/routes';

  import { APPS } from 'models/App';
  import { app } from 'modules/app/store';
  import BlockVisorFeature from 'modules/feature-flags/components/BlockVisorFeature/BlockVisorFeature.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    app.setActiveApp(APPS.BLOCKVISOR);
  });

  onMount(() => {
    setTimeout(() => {
      app.setBreadcrumbs([
        {
          title: 'Dashboard',
          url: ROUTES.DASHBOARD,
        },
        {
          title: 'Home',
          url: '',
        },
      ]);
    }, 200);
  });
</script>

<BlockVisorFeature>
  <slot />
</BlockVisorFeature>
