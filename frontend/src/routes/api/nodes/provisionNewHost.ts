import type { RequestHandler } from '@sveltejs/kit';
import { PROVISION_HOST } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const post: RequestHandler = async ({ request }) => {
  const { accessToken, refreshToken } = getTokens(request);
  try {
    const res = await httpClient.post(
      PROVISION_HOST,
      { org_id: '24f00a6c-1cb6-4660-8670-a9a7466699b2' },
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Refresh-Token': `${refreshToken}`,
        },
      },
    );

    return {
      status: res.status,
      body: {
        node: res.data,
      },
    };
  } catch (error) {
    return {
      status: error?.response?.status ?? 500,
      body: error?.response?.statusText,
    };
  }
};
