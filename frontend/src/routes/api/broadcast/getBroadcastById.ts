import type { RequestHandler } from '@sveltejs/kit';
import { SINGLE_BROADCAST } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request, url }) => {
  const { accessToken, refreshToken } = getTokens(request);

  const post_id = url.searchParams.get('post_id');

  try {
    const res = await httpClient.get(SINGLE_BROADCAST(post_id), {
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
    console.log(error);
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
