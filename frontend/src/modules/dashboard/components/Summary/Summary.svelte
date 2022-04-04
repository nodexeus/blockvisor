<script lang="ts">
  import StatsWithState from 'components/StatsWithState/StatsWithState.svelte';
  import IconBox from 'icons/box-12.svg';
</script>

<section class="summary">
  <StatsWithState changeOnZero={false} id="js-stats-nodes" value={49}>
    <IconBox />
    <span class="t-uppercase">Nodes</span>
  </StatsWithState>

  <StatsWithState state="active" id="js-stats-online" value={15}>
    <IconBox />
    <span class="t-uppercase">Online</span>
  </StatsWithState>

  <StatsWithState state="inactive" id="js-stats-offline" value={14}>
    <IconBox />
    <span class="t-uppercase">Offline</span>
  </StatsWithState>

  <StatsWithState id="js-stats-syncing" value={7}>
    <IconBox />
    <span class="t-uppercase">Syncing</span>
  </StatsWithState>

  <StatsWithState id="js-stats-consensus" value={13}>
    <IconBox />
    <span class="t-uppercase">Consensus</span>
  </StatsWithState>
</section>

<style>
  .summary {
    margin: 40px 0;
    display: flex;
    flex-wrap: wrap;
    gap: clamp(48px, 9vw, 60px);

    @media (--screen-medium) {
      & > :global(*:first-child) {
        padding-right: 32px;
        position: relative;

        &::after {
          content: '';
          position: absolute;
          top: 0;
          height: 76px;
          width: 1px;
          right: -16px;
          background-color: theme(--color-text-5-o10);
        }
      }
    }
  }
</style>
