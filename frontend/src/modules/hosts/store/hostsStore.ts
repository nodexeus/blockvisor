import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { HOSTS, SINGLE_HOST } from 'modules/authentication/const';
import { writable } from 'svelte/store';

export const hosts = writable([]);
export const selectedHosts = writable([]);

export const fetchAllHosts = async (user: UserSession) => {
  const all_hosts = [
    {
      title: 'All Hosts',
      href: ROUTES.HOSTS,
      children: [],
    },
  ];

  const res = await axios.get(HOSTS, {
    headers: { Authorization: `Bearer ${user ? user.token : ''}` },
  });

  const sorted = res.data.sort((a, b) => b.node_count - a.node_count);

  all_hosts[0].children = sorted.map((item) => {
    return {
      title: item.name,
      location: item.location,
      href: ROUTES.HOST_GROUP(item.token),
      id: item.id,
    };
  });

  hosts.set(all_hosts);
};

export const fetchHostById = async (id: string, user: UserSession) => {
  const res = await axios.get(SINGLE_HOST(id), {
    headers: { Authorization: `Bearer ${user ? user.token : ''}` },
  });

  console.log(res);

  selectedHosts.set(res.data);
};
