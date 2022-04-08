import { session } from '$app/stores';
import { derived } from 'svelte/store';

// Simple wrapper around session store for ease of use
export const user = derived(session, ($session) => {
  console.log($session);
  return $session.user;
});
