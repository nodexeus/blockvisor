<script lang="ts">
  import { APPS } from 'models/App';
  import { app } from 'modules/app/store';
  import BlockVisorFeature from 'modules/feature-flags/components/BlockVisorFeature/BlockVisorFeature.svelte';
  import { onMount } from 'svelte';
  import { pageTransition } from 'consts/animations';
  import LayoutTwoColumn from 'components/LayoutTwoColumn/LayoutTwoColumn.svelte';
  import HostsHierarchy from 'modules/hosts/components/HostsHierarchy/HostsHierarchy.svelte';
  import { ROUTES } from 'consts/routes';

  onMount(() => {
    app.setActiveApp(APPS.BLOCKVISOR);
  });

  const TEMP_NODES = [
    {
      title: 'All Hosts',
      href: ROUTES.HOSTS,
      children: [
        {
          title: 'Group 1',
          href: ROUTES.HOST_GROUP('id-1'),
          id: 'host-id1',
        },
        {
          title: 'Group 2',
          href: ROUTES.HOST_GROUP('id-2'),
          id: 'host-id2',
        },
        {
          title: 'Group 3',
          href: ROUTES.HOST_GROUP('id-3'),
          id: 'host-id3',
        },
      ],
    },
  ];
</script>

<BlockVisorFeature>
  <LayoutTwoColumn transition={pageTransition}>
    <HostsHierarchy slot="sidebar" nodes={TEMP_NODES} />
    <slot />
  </LayoutTwoColumn>
</BlockVisorFeature>
