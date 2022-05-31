import { API } from './api';

export const AUTHENTICATION = {
  LOGIN_POST: `${API.URL}/login`,
  REFRESH_TOKEN_POST: `${API.URL}/refresh`,
  USER_DETAILS_GET: `${API.URL}/whoami`,
};

export const HOSTS = {
  LISTS_HOSTS_GET: `${API.URL}/hosts`,
  ADD_HOST_POST: `${API.URL}/hosts`,
  GET_HOST_BY_TOKEN: (token: string) => `${API.URL}/hosts/token/${token}`,
  GET_HOST: (id: string) => `${API.URL}/hosts/${id}`,
  UPDATE_HOST: (id: string) => `${API.URL}/hosts/${id}`,
  DELETE_HOST: (id: string) => `${API.URL}/hosts/${id}`,
  LIST_COMMANDS_BY_HOST_GET: (id: string) => `${API.URL}/hosts/${id}/commands`,
  LIST_PENDING_COMMANDS_FOR_HOST_GET: (id: string) =>
    `${API.URL}/hosts/${id}/commands/pending`,
};

export const USERS = {
  CREATE_USER_POST: `${API.URL}/users`,
};

export const VALIDATORS = {
  LIST_VALIDATORS_GET: `${API.URL}/validators`,
  LIST_VALIDATORS_BY_USER_GET: (id: string) =>
    `${API.URL}/users/${id}/validators`,
  STAKE_VALIDATOR_POST: (id: string) => `${API.URL}/users/${id}/validators`,
  GET_VALIDATOR: (id: string) => `${API.URL}/users/validators/${id}`,
};

export const GROUPS = {
  GET_NODES_IN_GROUP: (id: string) => `${API.URL}/groups/nodes/${id}`,
  LIST_NODE_GROUPS_GET: `${API.URL}/groups/nodes`,
};

export const HOST_PROVISIONS = {
  CREATE_HOST_PROVISION_POST: `${API.URL}/host_provisions`,
  GET_HOST_PROVISION: (id: string) => `${API.URL}/host_provisions/${id}`,
  CREATE_HOST_FROM_PROVISION_POST: (id: string) =>
    `${API.URL}/host_provisions/${id}/hosts`,
};

export const NODES = {
  CREATE_NODE_POST: `${API.URL}/nodes`,
  GET_NODE: (id: string) => `${API.URL}/nodes/${id}`,
  UPDATE_NODE: (id: string) => `${API.URL}/nodes/${id}/info`,
};

export const BLOCKCHAINS = {
  LIST_BLOCKCHAINS_GET: `${API.URL}/blockchains`,
};

export const ORGANISATIONS = {
  LIST_USER_ORGANISATIONS_GET: (id: string) => `${API.URL}/users/${id}/orgs`,
};

export const BROADCAST = {
  LIST_BROADCAST_FILTERS_GET: (id: string) =>
    `${API.URL}/orgs/${id}/broadcast_filters`,
  GET_BROADCAST_FILTER: (id: string) => `${API.URL}/broadcast_filters/${id}`,
  CREATE_BROADCAST_FILTER: `${API.URL}/broadcast_filters`,
  UPDATE_BROADCAST_FILTER: (id: string) => `${API.URL}/broadcast_filters/${id}`,
  DELETE_BROADCAST_FILTER: (id: string) => `${API.URL}/broadcast_filters/${id}`,
};

export const ENDPOINTS = {
  AUTHENTICATION,
  HOSTS,
  USERS,
  VALIDATORS,
  GROUPS,
  HOST_PROVISIONS,
  NODES,
  BLOCKCHAINS,
  ORGANISATIONS,
  BROADCAST,
};
