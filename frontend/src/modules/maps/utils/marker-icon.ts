export const markerIcon = (L, marker) =>
  L.divIcon({
    html: `<span class="osm-map__marker">${marker}</span>`,
  });
