import { API } from 'consts/api';

export const CREATE_USER = `${API.URL}/users`;

export const GET_USER = `${API.URL}/whoami`;

export const REFRESH_TOKEN = `${API.URL}/refresh`;

export const LOGIN_USER = `${API.URL}/login`;

export const NODES = `${API.URL}/validators`;

export const NODE_GROUPS = `${API.URL}/groups/nodes`;

export const USER_NODES = (userId) => `${API.URL}/users/${userId}/validators`;
