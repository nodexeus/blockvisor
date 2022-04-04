import { browser } from '$app/env';

export const mapCleanup = (map) => {
  if (!browser || !map) {
    return;
  }

  map.remove();
  map = null;
};
