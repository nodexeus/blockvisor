import type { GetSession, Handle } from '@sveltejs/kit';
import axios from 'axios';
import cookie from 'cookie';
import { REFRESH_TOKEN } from 'modules/authentication/const';

export const getSession: GetSession = ({ locals }) => {
  return { user: locals.user, token: locals.token };
};

export const handle: Handle = async ({ event, resolve }) => {
  const cookies = cookie.parse(event.request.headers.get('cookie') || '');
  const refreshToken = cookies.refresh;

  if (!event.locals.token && refreshToken) {
    const res = await axios.post(REFRESH_TOKEN, {
      refresh: refreshToken,
    });

    const { refresh, token, ...user } = res.data;
    event.locals = { token, user: { ...user, verified: true } };
  }

  return await resolve(event);
};
