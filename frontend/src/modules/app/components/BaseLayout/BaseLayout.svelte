<script lang="ts">
  import { SvelteToast } from '@zerodevx/svelte-toast';
  import ChartProvider from 'modules/charts/components/ChartProvider/ChartProvider.svelte';
  import SkipToContent from '../SkipToContent/SkipToContent.svelte';
</script>

<SkipToContent />

<SvelteToast />

<ChartProvider />

<slot />

<style global>
  @import 'styles/app.pcss';

  :root {
    --toastBarHeight: 4px;
    --toastBarBackground: rgba(33, 36, 35, 0.2);
    --toastContainerTop: auto;
    --toastContainerRight: 14px;
    --toastContainerBottom: 8px;
    --toastContainerLeft: auto;
    --toastBorderRadius: 4px;
  }

  ._toastContainer {
    bottom: 0;
    right: 0;
    left: unset;
    top: unset;
  }

  ._toastMsg {
    font-size: 14px;
    line-height: 20px;
  }
</style>
