import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { derived, writable } from 'svelte/store';

export const nodes = writable([]);
export const selectedUser = writable();
export const selectedNode = writable([]);

export const fetchAllNodes = async (user: UserSession) => {
  const all_nodes = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [],
    },
  ];

  const res = await axios.get('/api/nodes/fetchNodes');

  const sorted = res.data.nodes.sort((a, b) => b.node_count - a.node_count);

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
  const res = await axios.get('/api/nodes/fetchNodeById', { params: { id } });

  selectedNode.set(res.data.node);
};

export const fetchUserById = async (id: string) => {
  const res = await axios.get('/api/nodes/fetchUserById', { params: { id } });

  selectedUser.set(res.data.user);
};

export const userDetails = (userId: string) =>
  derived(nodes, ($nodes) =>
    $nodes[0].children.find((item) => item.id === userId),
  );
