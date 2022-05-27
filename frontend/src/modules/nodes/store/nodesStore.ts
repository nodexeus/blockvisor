import { ENDPOINTS } from 'consts/endpoints';
import { ROUTES } from 'consts/routes';
import { derived, writable } from 'svelte/store';
import { httpClient } from 'utils/httpClient';

export const nodes = writable([]);
export const selectedUser = writable();
export const selectedNode = writable([]);
export const selectedValidator = writable({});
export const isLoading = writable(false);
export const installedNode = writable({});

export const fetchAllNodes = async () => {
  const all_nodes = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [],
    },
  ];

  const res = await httpClient.get(ENDPOINTS.GROUPS.LIST_NODE_GROUPS_GET);
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

export const fetchNodeById = async (id: string) => {
  const res = await httpClient.get(
    ENDPOINTS.VALIDATORS.LIST_VALIDATORS_BY_USER_GET(id),
  );

  selectedNode.set(res.data);
};

export const fetchUserById = async (id: string) => {
  const res = await httpClient.get(
    ENDPOINTS.VALIDATORS.LIST_VALIDATORS_BY_USER_GET(id),
  );

  const foundUser = res.data.find((item) => item.id === id);
  selectedUser.set(foundUser);
};

export const fetchValidatorById = async (id: string) => {
  isLoading.set(true);
  const res = await httpClient.get(ENDPOINTS.VALIDATORS.GET_VALIDATOR(id));
  selectedValidator.set(res.data);
  isLoading.set(false);
};

export const userDetails = (userId: string) =>
  derived(nodes, ($nodes) =>
    $nodes[0].children.find((item) => item.id === userId),
  );
