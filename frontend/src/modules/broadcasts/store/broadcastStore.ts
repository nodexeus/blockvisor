import axios from 'axios';
import { writable } from 'svelte/store';

export const blockchains = writable<Blockchain[]>([]);
export const organisationId = writable<string>();
export const broadcasts = writable<Broadcast[]>();

export const getAllBlockchains = async () => {
  axios.get('/api/nodes/getBlockchains').then((res) => {
    if (res.statusText === 'OK') {
      const active = res.data.filter(
        (item: Blockchain) => item.status === 'production',
      );
      const inactive = res.data.filter(
        (item: Blockchain) => item.status !== 'production',
      );
      blockchains.set([...active, ...inactive]);
    }
  });
};

export const getAllBroadcasts = async (orgId: string) => {
  return axios
    .get('/api/broadcast/getBroadcasts', {
      params: {
        org_id: orgId,
      },
    })
    .then((res) => {
      broadcasts.set(res.data);
    })
    .catch((err) => {
      blockchains.set([]);
      return err;
    });
};

export const getBroadcastById = async (postId: string) => {
  return axios
    .get('/api/broadcast/getBroadcastById', {
      params: {
        post_id: postId,
      },
    })
    .then((res) => {
      broadcasts.set(res.data);
    })
    .catch((err) => {
      blockchains.set([]);
      return err;
    });
};

export const getOrganisationId = async (userId: string) => {
  axios
    .get('/api/broadcast/getOrganisationId', { params: { id: userId } })
    .then((res) => {
      if (res.statusText === 'OK') {
        organisationId.set(res.data);
      }
    });
};
