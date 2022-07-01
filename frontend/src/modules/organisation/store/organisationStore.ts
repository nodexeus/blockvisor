import { toast } from 'components/Toast/Toast';
import { ENDPOINTS } from 'consts/endpoints';
import { get, writable } from 'svelte/store';
import { getUserInfo } from 'utils';
import { httpClient } from 'utils/httpClient';
import type { Organisation } from '../models/Organisation';

export const organisations = writable<Organisation[]>(undefined);
export const activeOrganisation = writable<Organisation>();
export const selectedOrganization = writable<Organisation>();
export const isLoading = writable(false);

export const getOrganisationById = async (orgId: string) => {
  isLoading.set(true);
  try {
    const res = await httpClient.get(
      ENDPOINTS.ORGANISATIONS.GET_ORGANISATION(orgId),
    );
    selectedOrganization.set(res.data);
    isLoading.set(false);
  } catch (error) {
    toast.warning('Something went wrong');
  }
};

export const getOrganisationsByUserId = async (userId: string) => {
  try {
    const res = await httpClient.get(
      ENDPOINTS.ORGANISATIONS.LIST_USER_ORGANISATIONS_GET(userId),
    );

    if (res.status === 200) {
      const orgs = res.data.map((item) => {
        if (item.is_personal && item.name === getUserInfo().email) {
          return {
            ...item,
            name: 'Personal Account',
          };
        }

        return item;
      });

      organisations.set(orgs);

      const privateOrg = orgs.find((item) => item.name === 'Personal Account');

      if (privateOrg) {
        activeOrganisation.set(privateOrg);
      } else {
        activeOrganisation.set(res.data[0]);
      }
    }
  } catch (error) {
    organisations.set([]);
  }
};

export const setActiveOrganisation = async (orgId: string) => {
  const orgs = get(organisations);

  const org = orgs.find((item) => item.id === orgId);

  activeOrganisation.set(org);
};
