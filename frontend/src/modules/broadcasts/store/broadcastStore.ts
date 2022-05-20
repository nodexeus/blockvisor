import axios from 'axios';
import { writable } from 'svelte/store';

export const blockchains = writable<Blockchain[]>([]);

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
