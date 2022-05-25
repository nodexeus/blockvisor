import type { RequestHandler } from '@sveltejs/kit';
import { ORGANISATIONS } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request, url }) => {
  const { accessToken, refreshToken } = getTokens(request);
  const id = url.searchParams.get('id');

  try {
    const res = await httpClient.get(ORGANISATIONS(id), {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Refresh-Token': `${refreshToken}`,
      },
    });

    if (res.statusText === 'OK') {
      return {
        status: res.status,
        body: res.data,
      };
    }

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
