import type { GetSession, Handle } from '@sveltejs/kit/types/hooks';
import { ROUTES } from 'consts/routes';
import cookie from 'cookie';

export const getSession: GetSession = (request) => {
  // TOOD: Implement user verification
  return {
    user: { ...JSON.parse(request.locals.user), verified: true },
  };
};

export const handle: Handle = async ({ event, resolve }) => {
  const cookies = cookie.parse(event.request.headers.get('cookie') || '');
  const isLogout = event.request.url.includes(ROUTES.AUTH_LOGOUT);
  event.locals.user = isLogout ? null : cookies?.user ?? null;
  const response = await resolve(event);

  if (event.locals.user) {
    response.headers.set(
      'set-cookie',
      cookie.serialize('user', event.locals.user, {
        path: '/',
        httpOnly: true,
      }),
    );
  }

  return response;
};
