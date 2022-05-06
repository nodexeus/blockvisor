<script lang="ts">
  import LayoutTwoColumn from 'components/LayoutTwoColumn/LayoutTwoColumn.svelte';
  import { pageTransition } from 'consts/animations';
  import { ROUTES } from 'consts/routes';

  import NodeHierarchy from 'modules/nodes/components/NodeHierarchy/NodeHierarchy.svelte';
</script>

<LayoutTwoColumn transition={pageTransition}>
  <NodeHierarchy slot="sidebar" />
  <slot />
</LayoutTwoColumn>
