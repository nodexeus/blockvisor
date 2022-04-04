// https://kit.svelte.dev/docs#hooks

import { authApi } from 'modules/authentication/service';
import { config } from 'modules/authentication/consts';
import type { GetSession, Handle } from '@sveltejs/kit/types/hooks';

export const getSession: GetSession = (request) => ({
  user: request.locals.session && {
    id: request.locals.session.identity.id,
    firstName: request.locals.session.identity.traits?.name?.first ?? '',
    lastName: request.locals.session.identity.traits?.name?.last ?? '',
    email: request.locals.session?.identity?.traits.email,
    verified:
      request.locals.session?.identity?.verifiable_addresses[0].verified ||
      false,
  },
});

export const handle: Handle = async ({ event, resolve }) => {
  try {
    const res = await authApi.toSession(undefined, 'session', {
      headers: {
        Authorization: `${event.request.headers.get('authorization')}`,
        Cookie: `${event.request.headers.get('cookie')}`,
        Origin: config.auth.publicUrl,
      },
      credentials: 'include',
    });

    const { data, status } = res;

    if (status === 401) {
      event.locals.session = undefined;
      return await resolve(event);
    }

    event.locals.session = data;

    const response = await resolve(event);

    return response;
  } catch (error) {
    return await resolve(event);
  }
};
