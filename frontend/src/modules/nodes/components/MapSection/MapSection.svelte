<script lang="ts">
  import { markerIcon } from 'modules/maps/utils';
  import { markerPin } from 'modules/maps/consts';
  import Map from 'modules/maps/components/Map/Map.svelte';
  import StatsWithLabel from 'components/StatsWithLabel/StatsWithLabel.svelte';

  const configureMap = (L, map: L.Map) => {
    const markerLayers = L.layerGroup();
    let icon = markerIcon(L, markerPin);
    let marker = L.marker([52.36, 4.9], { icon });
    markerLayers.addLayer(marker);
    markerLayers.addTo(map);
    map.setView([52.36, 4.9], 5);
  };
</script>

<article class="map-section">
  <h3 class="t-xlarge-fluid s-bottom--large">Map View</h3>
  <Map callback={configureMap} />

  <div class="map-section__wrapper">
    <ul class="u-list-reset map-section__stats">
      <li class="map-section__stats-item">
        <StatsWithLabel>
          <svelte:fragment slot="label">Lat</svelte:fragment>
          <svelte:fragment slot="value">38.8951</svelte:fragment>
        </StatsWithLabel>
      </li>
      <li class="map-section__stats-item">
        <StatsWithLabel>
          <svelte:fragment slot="label">Long</svelte:fragment>
          <svelte:fragment slot="value">-77.0364</svelte:fragment>
        </StatsWithLabel>
      </li>
      <li class="map-section__stats-item">
        <StatsWithLabel>
          <svelte:fragment slot="label">City</svelte:fragment>
          <svelte:fragment slot="value">Amsterdam</svelte:fragment>
        </StatsWithLabel>
      </li>
      <li class="map-section__stats-item">
        <StatsWithLabel>
          <svelte:fragment slot="label">Country</svelte:fragment>
          <svelte:fragment slot="value">Netherlands</svelte:fragment>
        </StatsWithLabel>
      </li>
    </ul>
  </div>
</article>

<style>
  .map-section {
    margin-top: 40px;
    padding-bottom: 40px;

    @media (--screen-medium) {
      margin-top: 80px;
      padding-bottom: 80px;
    }

    &__wrapper {
      overflow-x: auto;
      margin: 40px 0;
    }

    &__stats {
      display: flex;

      &-item {
        & + :global(.map-section__stats-item) {
          padding-left: 20px;
          border-left: 1px solid theme(--color-text-5-o10);
        }

        &:not(:last-child) {
          padding-right: 20px;
        }
      }
    }
  }
</style>
