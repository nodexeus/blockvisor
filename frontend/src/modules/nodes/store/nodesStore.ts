import axios from 'axios';
import { ROUTES } from 'consts/routes';
import {
  NODE_GROUPS,
  USER_NODES,
  VALIDATOR,
} from 'modules/authentication/const';
import { derived, writable } from 'svelte/store';

export const nodes = writable([]);
export const selectedUser = writable();
export const selectedNode = writable([]);
export const selectedValidator = writable({});
export const isLoading = writable(false);
export const installedNode = writable({});

export const fetchAllNodes = async (token: string) => {
  const all_nodes = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [],
    },
  ];

  const res = await axios.get(NODE_GROUPS, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  const sorted = res.data.sort((a, b) => b.node_count - a.node_count);

  all_nodes[0].children = sorted.map((item) => {
    return {
      title: item.name,
      href: ROUTES.NODE_GROUP(item.id),
      id: item.id,
    };
  });

  nodes.set(all_nodes);
};

export const fetchNodeById = async (id: string, token: string) => {
  const res = await axios.get(USER_NODES(id), {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  selectedNode.set(res.data);
};

export const fetchUserById = async (id: string, token: string) => {
  const res = await axios.get(USER_NODES(id), {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });

  const foundUser = res.data.find((item) => item.id === id);
  selectedUser.set(foundUser);
};

export const fetchValidatorById = async (id: string, token: string) => {
  isLoading.set(true);
  const res = await axios.get(VALIDATOR(id), {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  selectedValidator.set(res.data);
  isLoading.set(false);
};

export const userDetails = (userId: string) =>
  derived(nodes, ($nodes) =>
    $nodes[0].children.find((item) => item.id === userId),
  );
