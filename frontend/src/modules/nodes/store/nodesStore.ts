import axios from 'axios';
import { ROUTES } from 'consts/routes';
import { NODE_GROUPS, USER_NODES } from 'modules/authentication/const';
import { derived, writable } from 'svelte/store';

export const nodes = writable([]);
export const selectedUser = writable();
export const selectedNode = writable([]);

const token =
  'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiI4OWI3MzgyMi04ODM3LTQ5NTAtOTA4Yy0zZTNiM2E4MjJlMzQiLCJyb2xlIjoiYWRtaW4iLCJleHAiOjE2NTE5MjgxOTl9.G7NQPJoL-g97EKdplOC1DYUtlpi1Bs1rGE1fnyrKHOdhOS0F5kPqa7SYUgMJ7nQvBA4-IU283YY776JJxYQezA';

export const fetchAllNodes = async (user: UserSession) => {
  const all_nodes = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [],
    },
  ];

  const res = await axios.get(NODE_GROUPS, {
    headers: { Authorization: `Bearer ${user ? user.token : ''}` },
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

export const fetchNodeById = async (id: string, user: UserSession) => {
  const res = await axios.get(USER_NODES(id), {
    headers: { Authorization: `Bearer ${user ? user.token : ''}` },
  });

  selectedNode.set(res.data);
};

export const fetchUserById = async (id: string, user: UserSession) => {
  const res = await axios.get(NODE_GROUPS, {
    headers: { Authorization: `Bearer ${user ? user.token : ''}` },
  });

  const foundUser = res.data.find((item) => item.id === id);

  selectedUser.set(foundUser);
};

export const userDetails = (userId: string) =>
  derived(nodes, ($nodes) =>
    $nodes[0].children.find((item) => item.id === userId),
  );
