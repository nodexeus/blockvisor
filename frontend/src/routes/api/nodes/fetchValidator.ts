import type { RequestHandler } from '@sveltejs/kit';
import { VALIDATOR } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request, url }) => {
  const { accessToken, refreshToken } = getTokens(request);
  const id = url.searchParams.get('id');

  try {
    const res = await httpClient.get(VALIDATOR(id), {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'X-Refresh-Token': `${refreshToken}`,
      },
    });

    return {
      status: res.status,
      body: {
        validator: res.data,
      },
    };
  } catch (error) {
    console.log('ERRR', error);
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
