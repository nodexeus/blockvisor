import { session } from '$app/stores';
import { derived } from 'svelte/store';

// Simple wrapper around session store for easy use. Currently it returns mock user session so private routes remain accessible until auth has been implemented.
export const user = derived(session, ($session) => $session?.user ?? false);
