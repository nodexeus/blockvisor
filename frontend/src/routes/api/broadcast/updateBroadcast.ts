import type { RequestHandler } from '@sveltejs/kit';
import { SINGLE_BROADCAST } from 'modules/authentication/const';
import { httpClient } from 'utils/httpClient';
import { getTokens } from 'utils/ServerRequest';

export const post: RequestHandler = async ({ request }) => {
  const { accessToken, refreshToken } = getTokens(request);
  const data = await request.json();

  const {
    org_id,
    blockchain_id,
    callback_url,
    auth_token,
    addresses,
    name,
    txn_types,
  } = data.broadcast;

  console.log(data);

  try {
    const res = await httpClient.put(
      SINGLE_BROADCAST(data.id),
      {
        blockchain_id: blockchain_id,
        org_id: org_id,
        name: name,
        addresses: addresses,
        callback_url: callback_url,
        auth_token: auth_token,
        txn_types: txn_types,
        is_active: true,
      },
      {
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'X-Refresh-Token': `${refreshToken}`,
        },
      },
    );

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
