import axios from 'axios';
import {
  BLOCKCHAINS,
  BROADCASTS,
  SINGLE_BROADCAST,
} from 'modules/authentication/const';
import { writable } from 'svelte/store';
import { httpClient } from 'utils/httpClient';

export const blockchains = writable<Blockchain[]>([]);
export const broadcasts = writable<Broadcast[]>();

export const getAllBlockchains = async () => {
  try {
    const res = await httpClient.get(BLOCKCHAINS);

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
    const res = await httpClient.get(BROADCASTS(orgId));
    broadcasts.set(res.data);
  } catch (error) {
    blockchains.set([]);
  }
};

export const getBroadcastById = async (postId: string) => {
  try {
    const res = await httpClient.get(SINGLE_BROADCAST(postId));
    console.log(res.data);
    return res.data;
  } catch (error) {
    return error;
  }
};

export const deleteBroadcastById = async (postId: string, orgId: string) => {
  console.log('a');
  return httpClient
    .delete(SINGLE_BROADCAST(postId))
    .then((res) => {
      console.log(res);
      getAllBroadcasts(orgId).then((res) => res);
    })
    .catch((err) => {
      console.log(err);
      return err;
    });
};
