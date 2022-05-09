import type { RequestHandler } from '@sveltejs/kit';
import { NODE_GROUPS } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request, url }) => {
  const { accessToken, refreshToken } = getTokens(request);
  const id = url.searchParams.get('id');

  try {
    const res = await httpClient.get(NODE_GROUPS, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Refresh-Token': `${refreshToken}`,
      },
    });

    const foundUser = res.data.find((item) => item.id === id);

    return {
      status: res.status,
      body: {
        user: foundUser,
      },
    };
  } catch (error) {
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
