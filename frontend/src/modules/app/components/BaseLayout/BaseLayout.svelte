<script lang="ts">
  import { SvelteToast } from '@zerodevx/svelte-toast';
  import ChartProvider from 'modules/charts/components/ChartProvider/ChartProvider.svelte';
  import SkipToContent from '../SkipToContent/SkipToContent.svelte';
</script>

<SkipToContent />

<aside>
  <SvelteToast />
</aside>

<ChartProvider />

<slot />

<style global>
  @import 'styles/app.pcss';
</style>
