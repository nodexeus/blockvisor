import { ENDPOINTS } from 'consts/endpoints';
import { writable } from 'svelte/store';
import { httpClient } from 'utils/httpClient';

export const blockchains = writable<Blockchain[]>([]);
export const broadcasts = writable<Broadcast[]>();

export const getAllBlockchains = async () => {
  try {
    const res = await httpClient.get(
      ENDPOINTS.BLOCKCHAINS.LIST_BLOCKCHAINS_GET,
    );

    const active = res.data.filter(
      (item: Blockchain) => item.status === 'production',
    );
    const inactive = res.data.filter(
      (item: Blockchain) => item.status !== 'production',
    );
    blockchains.set([...active, ...inactive]);
  } catch (error) {
    blockchains.set([]);
  }
};

export const getAllBroadcasts = async (orgId: string) => {
  try {
    const res = await httpClient.get(
      ENDPOINTS.BROADCAST.LIST_BROADCAST_FILTERS_GET(orgId),
    );
    broadcasts.set(res.data);
  } catch (error) {
    blockchains.set([]);
  }
};

export const getBroadcastById = async (postId: string) => {
  try {
    const res = await httpClient.get(
      ENDPOINTS.BROADCAST.GET_BROADCAST_FILTER(postId),
    );
    return res.data;
  } catch (error) {
    return error;
  }
};

export const deleteBroadcastById = async (postId: string, orgId: string) => {
  return httpClient
    .delete(ENDPOINTS.BROADCAST.DELETE_BROADCAST_FILTER(postId))
    .then(() => {
      getAllBroadcasts(orgId).then((res) => res);
    })
    .catch((err) => {
      return err;
    });
};
function LIST_BLOCKCHAINS_GET(LIST_BLOCKCHAINS_GET: any) {
  throw new Error('Function not implemented.');
}
