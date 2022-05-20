<script lang="ts">
  import AlgoIcon from 'tokens/algo-24.svg';
  import AvaxIcon from 'tokens/avax-24.svg';
  import BtcIcon from 'tokens/btc-24.svg';
  import EthIcon from 'tokens/eth-24.svg';
  import HntIcon from 'tokens/hnt-24.svg';
  import SolIcon from 'tokens/sol-24.svg';

  const icons: { [key in TokenCode]: string } = {
    algo: AlgoIcon,
    avax: AvaxIcon,
    btc: BtcIcon,
    eth: EthIcon,
    hnt: HntIcon,
    sol: SolIcon,
  };

  export let icon = '';

  $: token = icons[icon];
</script>

<svelte:component this={token} />
