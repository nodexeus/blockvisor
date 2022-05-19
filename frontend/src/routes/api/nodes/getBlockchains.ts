import type { RequestHandler } from '@sveltejs/kit';
import { BLOCKCHAINS } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request }) => {
  const { accessToken, refreshToken } = getTokens(request);

  try {
    const res = await httpClient.get(BLOCKCHAINS, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Refresh-Token': `${refreshToken}`,
      },
    });

    return {
      status: res.status,
      body: res.data,
    };
  } catch (error) {
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
