<script lang="ts">
  import LayoutTwoColumn from 'components/LayoutTwoColumn/LayoutTwoColumn.svelte';
  import { pageTransition } from 'consts/animations';
  import { ROUTES } from 'consts/routes';

  import NodeHierarchy from 'modules/nodes/components/NodeHierarchy/NodeHierarchy.svelte';

  const TEMP_NODES = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [
        {
          id: '1',
          title: 'Group 1',
          href: ROUTES.NODE_GROUP('group-1'),
        },
        {
          id: '2',
          title: 'Group 2',
          href: ROUTES.NODE_GROUP('group-2'),
        },
        {
          id: '3',
          title: 'Group 3',
          href: ROUTES.NODE_GROUP('group-3'),
        },
      ],
    },
  ];
</script>

<LayoutTwoColumn transition={pageTransition}>
  <NodeHierarchy slot="sidebar" nodes={TEMP_NODES} />
  <slot />
</LayoutTwoColumn>
