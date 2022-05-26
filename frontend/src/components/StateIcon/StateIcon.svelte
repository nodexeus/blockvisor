<script lang="ts">
  import IconChain from 'icons/chain-12.svg';
  import IconCheckmark from 'icons/checkmark-12.svg';
  import IconConsensus from 'icons/consensus-12.svg';
  import IconFlag from 'icons/flag-12.svg';
  import IconExclamation from 'icons/exclamation-12.svg';
  import IconIndicator from 'icons/indicator-12.svg';
  import IconStopwatch from 'icons/stopwatch-12.svg';
  import IconUpdate from 'icons/update-12.svg';
  import IconRefresh from 'icons/refresh-12.svg';
  import type { StateIconProps } from './StateIcon.svelte';

  const icons: { [key in NodeState | HostState]: string } = {
    syncing: IconRefresh,
    synced: IconChain,
    upgrading: IconUpdate,
    consensus: IconConsensus,
    stopped: IconIndicator,
    pending: IconStopwatch,
    normal: IconCheckmark,
    loaded: IconFlag,
    issue: IconExclamation,
  };

  export let status: StateIconProps['status'];

  $: icon = icons[status];
</script>

<svelte:component this={icon} />
