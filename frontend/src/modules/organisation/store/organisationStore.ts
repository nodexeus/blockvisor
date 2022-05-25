import axios from 'axios';
import { writable } from 'svelte/store';
import type { Organisation } from '../models/Organisation';

export const organisations = writable<Organisation[]>();
export const activeOrganisation = writable<Organisation>();

export const getOrganisations = async (userId: string) => {
  axios
    .get('/api/organisation/getOrganisations', { params: { id: userId } })
    .then((res) => {
      if (res.statusText === 'OK') {
        organisations.set(res.data);

        const privateOrg = res.data.find((item) => item.is_personal);

        if (privateOrg) {
          activeOrganisation.set(privateOrg);
        } else {
          activeOrganisation.set(res.data[0]);
        }
      }
    });
};
