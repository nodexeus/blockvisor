import { ORGANISATIONS } from 'modules/authentication/const';
import { writable } from 'svelte/store';
import { httpClient } from 'utils/httpClient';
import type { Organisation } from '../models/Organisation';

export const organisations = writable<Organisation[]>();
export const activeOrganisation = writable<Organisation>();

export const getOrganisations = async (userId: string) => {
  try {
    const res = await httpClient.get(ORGANISATIONS(userId));

    organisations.set(res.data);

    const privateOrg = res.data.find((item) => item.is_personal);

    if (privateOrg) {
      activeOrganisation.set(privateOrg);
    } else {
      activeOrganisation.set(res.data[0]);
    }
  } catch (error) {
    organisations.set([]);
  }
};
