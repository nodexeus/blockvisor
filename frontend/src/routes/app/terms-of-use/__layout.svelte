<script>
  import StaticLayout from 'modules/app/components/StaticLayout/StaticLayout.svelte';
  import { onMount } from 'svelte';
  import { app } from 'modules/app/store';
  import { APPS } from 'models/App';

  onMount(() => {
    app.setActiveApp(APPS.BLOCKVISOR);
  });
</script>

<div class="grid grid-spacing container--large">
  <StaticLayout>
    <h2 slot="title" class="t-xxlarge-fluid">Terms of use</h2>
    <div class="t-color-text-3" slot="util">
      Last updated <date>June 28, 2021</date>
    </div>
    <slot />
  </StaticLayout>
</div>
