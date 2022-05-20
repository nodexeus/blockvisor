import { API } from 'consts/api';

export const CREATE_USER = `${API.URL}/users`;

export const GET_USER = `${API.URL}/whoami`;

export const REFRESH_TOKEN = `${API.URL}/refresh`;

export const LOGIN_USER = `${API.URL}/login`;

export const NODES = `${API.URL}/validators`;

export const HOSTS = `${API.URL}/hosts`;
export const SINGLE_HOST = (id: string) => `${API.URL}/hosts/${id}`;

export const HOST_GROUPS = (token: string) => `${API.URL}/hosts/token/${token}`;

export const NODE_GROUPS = `${API.URL}/groups/nodes`;

export const USER_NODES = (userId) => `${API.URL}/users/${userId}/validators`;

export const VALIDATOR = (id) => `${API.URL}/validators/${id}`;

export const PROVISION_HOST = `${API.URL}/host_provisions`;

export const CONFIRM_PROVISION = (host_id: string) =>
  `${API.URL}/host_provisions/${host_id}`;

export const INSTALL_NODE = `${API.URL}/nodes`;

export const BLOCKCHAINS = `${API.URL}/blockchains`;
export const ORGANISATIONS = (id: string) => `${API.URL}/users/${id}/orgs`;

export const CREATE_BROADCAST = `${API.URL}/broadcast_filters`;
