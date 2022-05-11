import type { RequestHandler } from '@sveltejs/kit';
import { NODE_GROUPS } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request }) => {
  const { accessToken, refreshToken } = getTokens(request);

  try {
    const res = await httpClient.get(NODE_GROUPS, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Refresh-Token': `${refreshToken}`,
      },
    });

    return {
      status: res.status,
      body: {
        nodes: res.data,
      },
    };
  } catch (error) {
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
