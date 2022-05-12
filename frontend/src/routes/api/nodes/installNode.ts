import type { RequestHandler } from '@sveltejs/kit';
import { PROVISION_HOST } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const post: RequestHandler = async ({ request }) => {
  const data = await request.formData();

  const { nodeType, host_id } = data;

  const { accessToken, refreshToken } = getTokens(request);
  try {
    const res = await httpClient.post(
      PROVISION_HOST,
      {
        org_id: '24f00a6c-1cb6-4660-8670-a9a7466699b2',
        host_id: host_id,
        chain_type: 'helium',
        node_type: nodeType.toString().toLowerCase(),
        status: 'installing',
        is_online: false,
      },
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Refresh-Token': `${refreshToken}`,
        },
      },
    );

    console.log(res);

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
