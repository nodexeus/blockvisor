import { writable } from 'svelte/store';
import type { Updater } from 'svelte/store';
import { APPS } from 'models/App';
import type { App } from '../models/App';

const INITIAL_STATE: App = {
  breadcrumbs: [],
  activeApp: APPS.BROADCAST,
  nodes: [],
};

const createActions = (
  set: (this: void, value: App) => void,
  update: (this: void, updater: Updater<App>) => void,
) => ({
  setActiveApp: (activeApp: App['activeApp']) =>
    update((store) => ({ ...store, activeApp })),
  setBreadcrumbs: (breadcrumbs: App['breadcrumbs']) =>
    update((store) => ({ ...store, breadcrumbs })),
  setNodes: (nodes: any) => update((store) => ({ ...store, nodes })),
  reset: () => set(INITIAL_STATE),
});

const createStore = () => {
  const { subscribe, set, update } = writable<App>(INITIAL_STATE);

  return {
    subscribe,
    ...createActions(set, update),
  };
};

export const app = createStore();
