import type { RequestHandler } from '@sveltejs/kit';
import { BROADCASTS } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request, url }) => {
  const { accessToken, refreshToken } = getTokens(request);

  const org_id = url.searchParams.get('org_id');

  try {
    const res = await httpClient.get(BROADCASTS(org_id), {
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
