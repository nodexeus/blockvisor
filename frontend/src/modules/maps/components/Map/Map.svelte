<script lang="ts">
  import { browser } from '$app/env';
  import { OSM_API_URL, OSM_ATTRIBUTION } from 'modules/maps/consts';
  import { mapCleanup } from 'modules/maps/utils/map-cleanup';
  import { onDestroy, onMount } from 'svelte';

  let map;
  let container;

  export let style = 'height:275px';
  export let callback;
  export let leafletOptions = { zoomControl: false };

  const initMapAndTiles = (L) => {
    map = L.map(container, leafletOptions).setView([0, 0], 5);

    L.tileLayer(OSM_API_URL, {
      attribution: OSM_ATTRIBUTION,
    }).addTo(map);
  };

  const initLeaflet = (L) => {
    initMapAndTiles(L);
    callback?.(L, map);
  };

  const handleMapResize = () => {
    if (!browser || !map) {
      return;
    }
    map.invalidateSize();
  };

  onMount(() => {
    if (!browser) {
      return;
    }

    window.addEventListener('resize', handleMapResize);
    import('leaflet').then(initLeaflet);
  });

  onDestroy(() => {
    if (!browser) {
      return;
    }

    window.removeEventListener('resize', handleMapResize);
    mapCleanup(map);
  });
</script>

<figure class="osm-map-container">
  <div bind:this={container} class="osm-map" {style} />
  <figcaption class="visually-hidden">
    <slot />
    Map for selected node
  </figcaption>
</figure>

<style global>
  @import '/node_modules/leaflet/dist/leaflet.css';

  .osm-map {
    background-color: transparent;
    font-family: var(--font-family-primary);

    &-container {
      max-width: 100%;
    }

    & :global(.leaflet-control-attribution) {
      @mixin font micro;
      background-color: theme(--color-text-1);
      color: theme(--color-text-4);

      & :global(a) {
        color: theme(--color-primary);
      }
    }

    & :global(.leaflet-tile-pane) {
      filter: invert(1) grayscale(1);
      opacity: 0.55;
    }

    & :global(.leaflet-div-icon) {
      background-color: transparent;
    }

    & :global(.osm-map__marker) {
      display: inline-block;
      width: 20px;
      color: var(--color-primary);
      transform: translateX(-50%) translateY(-50%);

      & :global(svg) {
        color: var(--color-primary);
      }
    }
  }
</style>
