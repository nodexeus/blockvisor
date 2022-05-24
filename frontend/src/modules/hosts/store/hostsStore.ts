import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { HOSTS, SINGLE_HOST } from 'modules/authentication/const';
import { writable } from 'svelte/store';

export const hosts = writable([]);
export const selectedHosts = writable([]);
export const provisionedHostId = writable('');
export const isLoading = writable(false);

export const fetchAllHosts = async (token: string) => {
  const all_hosts = [
    {
      title: 'All Hosts',
      href: ROUTES.HOSTS,
      children: [],
    },
  ];

  const res = await axios.get(HOSTS, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

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

export const fetchHostById = async (id: string, token: string) => {
  isLoading.set(true);
  const res = await axios.get(SINGLE_HOST(id), {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  isLoading.set(false);
  selectedHosts.set(res.data);
};

export const getHostById = async (id: string, token: string) => {
  const res = await axios.get(SINGLE_HOST(id), {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  return res.data;
};
