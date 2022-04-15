import { session } from '$app/stores';
import { derived } from 'svelte/store';
import type { Readable } from 'svelte/store';

export const user = derived<Readable<{ user: UserSession }>, UserSession>(
  session,
  ($session) => {
    return $session?.user ?? null;
  },
);
