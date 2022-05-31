import axios from 'axios';
import { ENDPOINTS } from 'consts/endpoints';
import { getUserInfo, saveUserinfo } from './localStorage';

const httpClient = axios.create();

httpClient.interceptors.request.use(
  (config) => {
    const userInfo = getUserInfo();
    config.headers = {
      Authorization: `Bearer ${userInfo.token}`,
    };
    return config;
  },
  (error) => {
    Promise.reject(error);
  },
);

httpClient.interceptors.response.use(
  async (response) => response,
  async (error) => {
    const originalRequest = error.config;
    if (error.response.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;
      const userInfo = getUserInfo();

      try {
        const res = await httpClient.post(
          ENDPOINTS.AUTHENTICATION.REFRESH_TOKEN_POST,
          {
            refresh: userInfo.refresh,
          },
        );
        const data = res.data;
        saveUserinfo({ ...res.data, verified: true });
        originalRequest.headers['Authorization'] = 'Bearer ' + data.token;

        return httpClient(originalRequest);
      } catch (error) {
        return Promise.reject(error);
      }
    }
    return Promise.reject(error);
  },
);
export { httpClient };
