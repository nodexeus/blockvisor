import type { RequestHandler } from '@sveltejs/kit';
import { INSTALL_NODE } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const get: RequestHandler = async ({ request, url }) => {
  const { accessToken, refreshToken } = getTokens(request);
  const nodeType = url.searchParams.get('node_type');
  const hostId = url.searchParams.get('host_id');

  let realNodeType;

  switch (nodeType) {
    case 'ETL':
      realNodeType = 'etl';
      break;
    case 'Node/api':
      realNodeType = 'node';
      break;
    case 'Validator':
      realNodeType = 'validator';
      break;
  }

  try {
    const res = await httpClient.post(
      INSTALL_NODE,
      {
        org_id: '24f00a6c-1cb6-4660-8670-a9a7466699b2',
        host_id: hostId,
        chain_type: 'helium',
        node_type: realNodeType,
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
