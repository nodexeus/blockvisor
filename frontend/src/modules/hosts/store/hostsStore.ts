import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { HOSTS, SINGLE_HOST } from 'modules/authentication/const';
import { writable } from 'svelte/store';
import { httpClient } from 'utils/httpClient';

export const hosts = writable([]);
export const selectedHosts = writable([]);
export const provisionedHostId = writable('');
export const isLoading = writable(false);

export const fetchAllHosts = async () => {
  const all_hosts = [
    {
      title: 'All Hosts',
      href: ROUTES.HOSTS,
      children: [],
    },
  ];

  const res = await httpClient.get(HOSTS);

  const sorted = res.data.sort((a, b) => b.node_count - a.node_count);

  all_hosts[0].children = sorted.map((item) => {
    return {
      title: item.name,
      location: item.location,
      href: ROUTES.HOST_GROUP(item.id),
      id: item.id,
      ...item,
    };
  });

  hosts.set(all_hosts);
};

export const fetchHostById = async (id: string) => {
  isLoading.set(true);
  const res = await httpClient.get(SINGLE_HOST(id));

  isLoading.set(false);
  selectedHosts.set(res.data);
};

export const getHostById = async (id: string) => {
  const res = await httpClient.get(SINGLE_HOST(id));

  return res.data;
};
